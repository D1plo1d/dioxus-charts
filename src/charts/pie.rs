use dioxus::prelude::*;

use crate::types::{Labels, Point};
use crate::utils::{normalize_series, polar_to_cartesian};

/// A hint for the automatic positioning of labels in the pie chart.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LabelPosition {
    /// To position the label inside the pie chart.
    Inside,
    /// To position the label outside close to the border of the pie chart.
    Outside,
    /// To position the label in the center for manually positioning with the `label_offset` prop.
    Center,
}

/// The `PieChart` properties struct for the configuration of the pie chart.
#[derive(PartialEq, Props)]
pub struct PieChartProps<'a> {
    series: Vec<f32>,
    #[props(optional)]
    labels: Option<Labels>,

    #[props(default = "100%")]
    width: &'a str,
    #[props(default = "100%")]
    height: &'a str,
    #[props(default = 600)]
    viewbox_width: i32,
    #[props(default = 400)]
    viewbox_height: i32,

    #[props(default = true)]
    show_labels: bool,
    #[props(default=LabelPosition::Inside)]
    label_position: LabelPosition,
    #[props(default)]
    label_offset: f32,
    #[props(optional)]
    label_interpolation: Option<fn(f32) -> String>,

    #[props(default)]
    start_angle: f32,
    #[props(optional)]
    total: Option<f32>,
    #[props(optional)]
    show_ratio: Option<f32>,
    #[props(default)]
    padding: f32,

    #[props(default = false)]
    donut: bool,
    #[props(default = 40.0)]
    donut_width: f32,

    #[props(default = "dx-pie-chart")]
    class_chart: &'a str,
    #[props(default = "dx-series")]
    class_series: &'a str,
    #[props(default = "dx-slice")]
    class_slice: &'a str,
    #[props(default = "dx-label")]
    class_label: &'a str,
}

/// This is the `PieChart` function used to render the pie chart `Element`.
/// In Dioxus, components are just functions, so this is the main `PieChart`
/// component to be used inside `rsx!` macros in your code.
///
/// # Example
///
/// ```rust,ignore
/// use dioxus::prelude::*;
/// use dioxus_charts::PieChart;
///
/// fn app(cx: Scope) -> Element {
///     cx.render(rsx! {
///         PieChart {
///             start_angle: -60.0,
///             label_position: LabelPosition::Outside,
///             label_offset: 35.0,
///             padding: 20.0,
///             series: vec![59.54, 17.2, 9.59, 7.6, 5.53, 0.55]
///             labels: vec!["Asia".into(), "Africa".into(), "Europe".into(), "N. America".into(), "S. America".into(), "Oceania".into()],
///         }
///     })
/// }
/// ```
///
/// # Props
///
/// - `series`: [Vec]<[f32]> (**required**): The series vector with the values.
/// - `labels`: [Vec]<[String]> (optional): Optional labels to show for each value of the
/// series.
/// ---
/// - `width`: &[str] (default: `"100%"`): The SVG element width attribute. It also accepts any
/// other CSS style, i.e., "200px"
/// - `height`: &[str] (default: `"100%"`): The SVG height counter-part of the `width` prop above.
/// - `viewbox_width`: [i32] (default: `600`): The SVG viewbox width. Together with
/// `viewbox_height` it is useful scaling up or down the chart and labels.
/// - `viewbox_height`: [i32] (default: `400`): The SVG viewbox height.
/// ---
/// - `show_labels`: [bool] (default: `true`): Show/hide labels.
/// - `label_position`: [`LabelPosition`] (default: [`LabelPosition::Inside`]): A hint for the
/// automatic positioning of labels on the chart.
/// - `label_offset`: [f32] (default: `0.0`): An extra offset for the labels relative to the center
/// of the pie.
/// - `label_interpolation`: fn([f32]) -> [String] (optional): Function for formatting the
/// generated labels.
/// ---
/// - `start_angle`: [f32] (default: `0.0`): The initial angle used for drawing the pie.
/// - `total`: [f32] (optional): The series total sum. Can be used to make Gauge charts.
/// - `show_ratio`: [f32] (optional): Used for making Gauge charts more easily. `0.0001` to
/// `1.0` is the same as `0%` to `100%`.
/// - `padding`: [f32] (default: `0.0`): Padding for every side of the SVG view box.
/// ---
/// - `donut`: [bool] (default: `false`): Draw the slices differently to make a donut-looking chart
/// instead.
/// - `donut_width`: [f32] (default: `40.0`): The width of each donut slice.
/// ---
/// - `class_chart`: &[str] (default: `"dx-pie-chart"`): The HTML element `class` of the
/// pie chart.
/// - `class_series`: &[str] (default: `"dx-series"`): The HTML element `class` for the group of
/// pie slices.
/// - `class_slice`: &[str] (default: `"dx-slice"`): The HTML element `class` for all pie
/// slices.
/// - `class_label`: &[str] (default: `"dx-label"`): The HTML element `class` for all labels.
#[allow(non_snake_case)]
pub fn PieChart<'a>(cx: Scope<'a, PieChartProps<'a>>) -> Element {
    if cx.props.series.is_empty() {
        return cx.render(rsx!("Pie chart error: empty series"));
    }

    let center = Point::new(
        cx.props.viewbox_width as f32 / 2.0,
        cx.props.viewbox_height as f32 / 2.0,
    );
    let center_min = center.x.min(center.y);
    let radius = center_min - 30.0 - cx.props.padding;
    let label_radius = match cx.props.label_position {
        LabelPosition::Inside => radius / 2.0 + cx.props.label_offset,
        LabelPosition::Outside => radius + cx.props.label_offset,
        LabelPosition::Center => 0.0 + cx.props.label_offset,
    };

    let normalized_series = normalize_series(&cx.props.series);
    let normalized_sum: f32 = normalized_series.iter().sum();

    let values_total: f32 = if let Some(r) = cx.props.show_ratio {
        1.0 / r.clamp(0.0001, 1.0) * normalized_sum
    } else if let Some(v) = cx.props.total {
        (normalized_sum / cx.props.series.iter().sum::<f32>() * v).max(normalized_sum)
    } else {
        normalized_sum
    };

    let mut m_start_angle = cx.props.start_angle;
    let mut color_var = 255.0;
    let mut class_index = 0;
    let mut label_positions = Vec::<Point>::new();

    cx.render(rsx! {
        div {
            svg {
                view_box: "0 0 {cx.props.viewbox_width} {cx.props.viewbox_height}",
                width: "{cx.props.width}",
                height: "{cx.props.height}",
                class: "{cx.props.class_chart}",
                preserveAspectRatio: "xMidYMid meet",
                xmlns: "http://www.w3.org/2000/svg",
                normalized_series.iter().map(|v| {
                    if *v != 0.0 {
                        let mut end_angle = if values_total > 0.0 {
                            m_start_angle + (v / values_total) * 360.0
                        } else {
                            0.0
                        };
                        let overlap_start_angle = if class_index != 0 {
                            (m_start_angle - 0.4).max(0.0)
                        } else {
                            m_start_angle
                        };
                        if end_angle - overlap_start_angle >= 359.99 {
                            end_angle = overlap_start_angle + 359.99
                        }

                        let start_position = polar_to_cartesian(center, radius, overlap_start_angle);
                        let end_position = polar_to_cartesian(center, radius, end_angle);
                        let large_arc = i32::from(end_angle - m_start_angle > 180.0);

                        let dpath = if cx.props.donut {
                            let donut_radius = radius - cx.props.donut_width;
                            let start_inside_position = polar_to_cartesian(center, donut_radius, overlap_start_angle);
                            let end_inside_position = polar_to_cartesian(center, donut_radius, end_angle);
                            let large_arc_inside = large_arc;

                            format!("M{end_position}\
                                     A{radius},{radius},0,{large_arc},0,{start_position}\
                                     L{start_inside_position}\
                                     A{donut_radius},{donut_radius},0,{large_arc_inside},1,{end_inside_position}Z")
                        } else {
                            format!("M{end_position}\
                                     A{radius},{radius},0,{large_arc},0,{start_position}\
                                     L{center}Z")
                        };

                        let element = rsx! {cx,
                            g {
                                key: "{class_index}",
                                class: "{cx.props.class_series} {cx.props.class_series}-{class_index}",
                                path {
                                    d: "{dpath}",
                                    class: "{cx.props.class_slice}",
                                    fill: "rgb({color_var}, 40, 40)",
                                },
                            }
                        };

                        label_positions.push(polar_to_cartesian(center, label_radius, m_start_angle + (end_angle - m_start_angle) / 2.0));

                        color_var -= 75.0 * (1.0 / (class_index + 1) as f32);
                        class_index += 1;
                        m_start_angle = end_angle;
                        element
                    } else {
                        label_positions.push(Point::new(-1.0, -1.0));
                        None
                    }
                }),
                if let Some(ref labels) = cx.props.labels {
                    rsx! {cx,
                        g {
                            label_positions.iter().zip(labels.iter()).map(|(position, label)| {
                                if position.x > 0.0 {
                                    rsx! {cx,
                                        text {
                                            key: "{label}",
                                            dx: "{position.x}",
                                            dy: "{position.y}",
                                            text_anchor: "middle",
                                            class: "{cx.props.class_label}",
                                            alignment_baseline: "middle",
                                            [label.as_str()]
                                        }
                                    }
                                } else {
                                    None
                                }
                            })
                        }
                    }
                } else if cx.props.show_labels {
                    rsx! {cx,
                        g {
                            label_positions.iter().zip(cx.props.series.iter()).map(|(position, value)| {
                                let label = if let Some(func) = cx.props.label_interpolation {
                                    func(*value)
                                } else {
                                    value.to_string()
                                };

                                if position.x > 0.0 {
                                    rsx! {cx,
                                        text {
                                            key: "label",
                                            dx: "{position.x}",
                                            dy: "{position.y}",
                                            text_anchor: "middle",
                                            class: "{cx.props.class_label}",
                                            alignment_baseline: "middle",
                                            "{label}"
                                        }
                                    }
                                } else {
                                    None
                                }
                            })
                        }
                    }
                } else {
                    None
                }
            }
        }
    })
}
