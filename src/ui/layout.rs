use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    widgets::canvas::{Canvas, Line, Map, MapResolution, Rectangle},
    widgets::{
        Axis, BarChart, Block, Borders, Chart, Dataset, Gauge, List, Paragraph, Row, Sparkline,
        Table, Tabs, Text,
    },
    Frame,
};

use crate::quotes::get_quote;
use crate::services::{systemd::*, get_docker_processes};
use crate::sys::{get_all_disks, MemUnit};
use crate::ui::App;

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
        .split(f.size());
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Header")
        .title_style(Style::default().fg(Color::Gray).modifier(Modifier::BOLD));
    let text = [
        Text::raw("Welcome to "),
        Text::styled(
            "Magni",
            Style::default()
                .fg(Color::Rgb(109, 80, 168))
                .modifier(Modifier::BOLD),
        ),
        Text::styled(
            "Linux",
            Style::default()
                .fg(Color::Rgb(255, 255, 255))
                .modifier(Modifier::BOLD),
        ),
    ];
    let paragraph = Paragraph::new(text.iter()).block(block).wrap(true);
    f.render_widget(paragraph, chunks[0]);
    draw_first_tab(f, app, chunks[1]);
}

fn draw_first_tab<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let docker = get_docker_processes();
    let chunks = Layout::default()
        .constraints(
            [
                Constraint::Length((get_all_disks().len() * 2) as u16 + 2),
                Constraint::Min(7),
                Constraint::Length(7),
            ]
            .as_ref(),
        )
        .split(area);
    draw_gauges(f, app, chunks[0]);
    draw_charts(f, app, chunks[1],  docker);
    draw_text(f, chunks[2]);
}

fn draw_gauges<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let disks = get_all_disks();
    let mut c: usize = 0;
    let chunks = match get_all_disks().len() {
        2 => {
            Layout::default()
            .constraints([Constraint::Length(2), Constraint::Length(2)].as_ref())
            .margin(1)
            .split(area)
        },
        3 => {
            Layout::default()
            .constraints([Constraint::Length(2), Constraint::Length(2), Constraint::Length(2)].as_ref())
            .margin(1)
            .split(area)
        },
        4 => {
            Layout::default()
            .constraints([Constraint::Length(2), Constraint::Length(2), Constraint::Length(2), Constraint::Length(2)].as_ref())
            .margin(1)
            .split(area)
        },
        5 => {
            Layout::default()
            .constraints([Constraint::Length(2), Constraint::Length(2), Constraint::Length(2), Constraint::Length(2), Constraint::Length(2)].as_ref())
            .margin(1)
            .split(area)
        },
        _ => {
            Layout::default()
            .constraints([Constraint::Length(2)].as_ref())
            .margin(1)
            .split(area)
        }
    };
    let block = Block::default().borders(Borders::ALL).title("Drives");
    f.render_widget(block, area);
    for disk in disks {
        let disk_perc =
            (((disk.total_space - disk.available_space) as f64 / disk.total_space as f64) * 100.0)
                .ceil();
        let gauge_title = String::from("Mount Path: ") + disk.mount_point.to_str().unwrap();
        let label = match disk.total_space {
            1_000..=999_999 => {
                disk_perc.to_string()
                    + "% out of "
                    + &format!("{}", MemUnit::KB(disk.total_space as f64))
            }
            1_000_000..=999_999_999 => {
                disk_perc.to_string()
                    + "% out of "
                    + &format!("{}", MemUnit::MB(disk.total_space as f64))
            }
            1_000_000_000..=999_999_999_999 => {
                disk_perc.to_string()
                    + "% out of "
                    + &format!("{}", MemUnit::GB(disk.total_space as f64))
            }
            1_000_000_000_000..=999_999_999_999_000 => {
                disk_perc.to_string()
                    + "% out of "
                    + &format!("{}", MemUnit::TB(disk.total_space as f64))
            }
            1_000_000_000_000_000..=999_999_999_999_999_999 => {
                disk_perc.to_string()
                    + "% out of "
                    + &format!("{}", MemUnit::PB(disk.total_space as f64))
            }
            _ => {
                disk_perc.to_string()
                    + "% out of "
                    + &format!("{}", MemUnit::B(disk.total_space as f64))
            }
        };
        //let label = format!("{}", disk_perc);
        let gauge = Gauge::default()
            .block(Block::default().title(&gauge_title))
            .style(
                Style::default()
                    .fg(Color::Magenta)
                    .bg(Color::Black)
                    .modifier(Modifier::ITALIC | Modifier::BOLD),
            )
            .label(&label)
            .percent(disk_perc as u16);
        f.render_widget(gauge, chunks[c]);
        c += 1;
    }
}

fn draw_charts<B>(f: &mut Frame<B>, app: &mut App, area: Rect, docker: Option<Vec<(String, String)>>)
where
    B: Backend,
{
    let constraints = if app.show_chart {
        vec![Constraint::Percentage(50), Constraint::Percentage(50)]
    } else {
        vec![Constraint::Percentage(100)]
    };
    let chunks = Layout::default()
        .constraints(constraints)
        .direction(Direction::Horizontal)
        .split(area);
    {
        let chunks = Layout::default()
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(chunks[0]);
        {
            let chunks = Layout::default()
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .direction(Direction::Horizontal)
                .split(chunks[0]);

            // Colors
            let info_style = Style::default().fg(Color::White);
            let warning_style = Style::default().fg(Color::Yellow);
            let error_style = Style::default().fg(Color::Magenta);
            let critical_style = Style::default().fg(Color::Red).modifier(Modifier::BOLD);
            let success_style = Style::default().fg(Color::Green).modifier(Modifier::BOLD);

            // List Systemd units
            let units = app.wanted_systemd_units.iter().map(|SystemdUnit { name, state }| {
                Text::styled(
                    format!("{}: {}", name, state),
                    match state {
                        UnitState::Bad
                        | UnitState::Disabled
                        | UnitState::Masked
                        | UnitState::MaskedRuntime => critical_style,
                        UnitState::Enabled | UnitState::EnabledRuntime => success_style,
                        _ => info_style,
                    },
                )
            });
            
            let units =
                List::new(units).block(Block::default().borders(Borders::ALL).title("Systemd"));
            f.render_stateful_widget(units, chunks[0], &mut app.tasks.state);

            // List Docker items
            match docker {
                Some(x) => {
                    let up_style = Style::default().fg(Color::Green);
                    let failure_style = Style::default()
                        .fg(Color::Red)
                        .modifier(Modifier::RAPID_BLINK | Modifier::CROSSED_OUT);
                    let header = ["Image", "Status"];
                    let rows = x.iter().map(|(i , s)| {
                        let style = if s.contains("Up") {
                            up_style
                        } else {
                            failure_style
                        };
                        Row::StyledData(vec![i, s].into_iter(), style)
                    });
                    let table = Table::new(header.iter(), rows)
                        .block(Block::default().title("Docker").borders(Borders::ALL))
                        .header_style(Style::default().fg(Color::Yellow))
                        .widths(&[
                            Constraint::Length(15),
                            Constraint::Length(15),
                            Constraint::Length(10),
                        ]);
                    f.render_widget(table, chunks[1]);
                },
                None => {
                    let text = [
                        Text::raw(" "),
                    ];
                    let block = Block::default()
                        .borders(Borders::ALL)
                        .title("Empty")
                        .title_style(Style::default().fg(Color::Gray).modifier(Modifier::BOLD));
                    let paragraph = Paragraph::new(text.iter()).block(block).wrap(true);
                    f.render_widget(paragraph, chunks[1]);
                }
            }
        }

        // Create Bar graph with 
        // Memory Usage
        // Swap Usage
        // Load Avg
        // Proc Root / Proc User
        // Network Connections per address
        let barchart = BarChart::default()
            .block(Block::default().borders(Borders::ALL).title("Stats"))
            .data(&app.barchart)
            .bar_width(9)
            .bar_gap(1)
            .bar_set(if app.enhanced_graphics {
                symbols::bar::NINE_LEVELS
            } else {
                symbols::bar::THREE_LEVELS
            })
            .value_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Magenta)
                    .modifier(Modifier::ITALIC),
            )
            .label_style(Style::default().fg(Color::Blue))
            .style(Style::default().fg(Color::Magenta));
        f.render_widget(barchart, chunks[1]);
    }
    if app.show_chart {
        let x_labels = [
            format!("{}", app.signals.window[0]),
            format!("{}", (app.signals.window[0] + app.signals.window[1]) / 2.0),
            format!("{}", app.signals.window[1]),
        ];
        let datasets = [
            Dataset::default()
                .name("data2")
                .marker(symbols::Marker::Dot)
                .style(Style::default().fg(Color::Cyan))
                .data(&app.signals.sin1.points),
            Dataset::default()
                .name("data3")
                .marker(if app.enhanced_graphics {
                    symbols::Marker::Braille
                } else {
                    symbols::Marker::Dot
                })
                .style(Style::default().fg(Color::Yellow))
                .data(&app.signals.sin2.points),
        ];
        let chart = Chart::default()
            .block(
                Block::default()
                    .title("Chart")
                    .title_style(Style::default().fg(Color::Cyan).modifier(Modifier::BOLD))
                    .borders(Borders::ALL),
            )
            .x_axis(
                Axis::default()
                    .title("X Axis")
                    .style(Style::default().fg(Color::Gray))
                    .labels_style(Style::default().modifier(Modifier::ITALIC))
                    .bounds(app.signals.window)
                    .labels(&x_labels),
            )
            .y_axis(
                Axis::default()
                    .title("Y Axis")
                    .style(Style::default().fg(Color::Gray))
                    .labels_style(Style::default().modifier(Modifier::ITALIC))
                    .bounds([-20.0, 20.0])
                    .labels(&["-20", "0", "20"]),
            )
            .datasets(&datasets);
        f.render_widget(chart, chunks[1]);
    }
}

fn draw_text<B>(f: &mut Frame<B>, area: Rect)
where
    B: Backend,
{
    let quote = get_quote();
    let text = [
        Text::raw(quote.quote),
        Text::raw("\n\n -  "),
        Text::styled(quote.author, Style::default().modifier(Modifier::BOLD)),
    ];
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Quote")
        .title_style(Style::default().fg(Color::Gray).modifier(Modifier::BOLD));
    let paragraph = Paragraph::new(text.iter()).block(block).wrap(true);
    f.render_widget(paragraph, area);
}
