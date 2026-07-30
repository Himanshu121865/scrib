use scrib::point::Point;
use scrib::stroke;

fn main() {
    let raw = generate_points();

    let s = stroke::Stroke::new_with_points(raw.clone());
    let output = s.process_with_widths(4.0, 0.75);

    let svg = render_svg(&raw, &output);
    println!("{svg}");
}

fn generate_points() -> Vec<Point> {
    let mut pts = Vec::new();
    for i in 0..80 {
        let t = i as f64 * 0.15;
        let x = 50.0 + t * 8.0;
        let y = 200.0 + (t * 0.8).sin() * 60.0 + (t * 0.3).cos() * 20.0;
        let pressure = 0.3 + ((i as f64 / 80.0) * 0.7);
        pts.push(Point::new(x, y, pressure));
    }
    pts
}

fn render_svg(raw: &[Point], output: &[[f64; 3]]) -> String {
    let raw_pts: String = raw
        .iter()
        .map(|p| {
            format!(
                "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"1.5\" fill=\"#aaa\"/>",
                p.x, p.y
            )
        })
        .collect::<Vec<_>>()
        .join("\n    ");

    let path_d: String = output
        .iter()
        .enumerate()
        .map(|(i, p)| {
            if i == 0 {
                format!("M {:.1} {:.1}", p[0], p[1])
            } else {
                format!("L {:.1} {:.1}", p[0], p[1])
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    let width_pts: String = output
        .iter()
        .map(|p| {
            format!(
                "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"{:.1}\" fill=\"none\" stroke=\"#080\" stroke-width=\"0.5\"/>",
                p[0], p[1], p[2] / 2.0
            )
        })
        .collect::<Vec<_>>()
        .join("\n    ");

    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 700 400\" width=\"700\" height=\"400\">\n\
         <rect width=\"700\" height=\"400\" fill=\"#fff\"/>\n\
         <g fill=\"none\" stroke=\"#222\" stroke-width=\"3\" stroke-linecap=\"round\" stroke-linejoin=\"round\">\n\
         <path d=\"{path_d}\"/>\n\
         </g>\n\
         {raw_pts}\n\
         {width_pts}\n\
         <text x=\"20\" y=\"30\" font-family=\"monospace\" font-size=\"13\" fill=\"#888\">\n\
         gray = raw | black = smoothed path | green circles = variable width\n\
         </text>\n\
         </svg>",
    )
}
