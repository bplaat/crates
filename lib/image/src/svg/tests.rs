/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::collections::HashSet;
use std::fs;

use super::path::parse_path;
use super::xml::XmlDocument;
use super::{decode, is_svg};
use crate::test_support::FixtureCorpus;
use crate::{Color, DrawCommand, MaskType, Paint, PathSegment, Point, VectorDecodeError};

fn match_reference(source: &str) -> String {
    let document = XmlDocument::parse(source).expect("parse WPT SVG metadata");
    let mut references = document.elements.iter().filter_map(|element| {
        (element.name == "link" && element.attr("rel") == Some("match"))
            .then(|| element.attr("href"))
            .flatten()
    });
    let reference = references.next().expect("WPT SVG has a match reference");
    assert!(
        references.next().is_none(),
        "WPT SVG has exactly one match reference"
    );
    reference.to_owned()
}

#[test]
fn decodes_wpt_reference_pairs_offline() {
    let corpus = FixtureCorpus::new("svg");
    let canonical_reference_root = corpus
        .references
        .canonicalize()
        .expect("canonical SVG reference directory");
    let files = corpus.image_files("svg");
    assert_eq!(files.len(), 93, "pinned WPT SVG test count changed");
    let mut used_references = HashSet::new();
    for path in files {
        let relative = corpus.relative_name(&path);
        let data = fs::read(&path).expect("read SVG fixture");
        let source = std::str::from_utf8(&data).expect("WPT SVG is UTF-8");
        let reference = path
            .parent()
            .expect("SVG test directory")
            .join(match_reference(source))
            .canonicalize()
            .expect("resolve SVG match reference");
        assert!(
            reference.starts_with(&canonical_reference_root),
            "{}: match reference must be under tests/reference/svg",
            relative.display()
        );
        used_references.insert(
            reference
                .strip_prefix(&canonical_reference_root)
                .expect("relative canonical SVG reference")
                .to_owned(),
        );

        let image = crate::decode_vector(&data)
            .unwrap_or_else(|error| panic!("{}: {error}", relative.display()));
        let reference_data = fs::read(&reference).expect("read SVG match reference");
        let reference_image = crate::decode_vector(&reference_data)
            .unwrap_or_else(|error| panic!("{} reference: {error}", relative.display()));
        assert_eq!(
            image.size(),
            reference_image.size(),
            "{}",
            relative.display()
        );
    }

    let references = corpus.reference_files("svg");
    assert_eq!(references.len(), 41, "pinned SVG reference count changed");
    let references = references
        .into_iter()
        .map(|reference| {
            reference
                .strip_prefix(&corpus.references)
                .expect("relative SVG reference")
                .to_owned()
        })
        .collect();
    assert_eq!(used_references, references);
}

#[test]
fn decodes_compound_vector_artwork() {
    let image = decode(
        br##"<svg xmlns="http://www.w3.org/2000/svg" width="60" height="40">
          <path d="M2 2h20v20H2z" fill="#f60"/>
          <g transform="translate(30 5)">
            <path d="M0 15C0 5 20 5 20 15Z" fill="#06f"/>
          </g>
        </svg>"##,
    )
    .expect("inline SVG should decode");
    assert_eq!(image.size().width, 60.0);
    assert_eq!(
        image
            .commands()
            .iter()
            .filter(|command| matches!(command, DrawCommand::Fill { .. }))
            .count(),
        2
    );
}

#[test]
fn parses_compact_path_numbers_and_arcs() {
    let path = parse_path("M.5-.5L1e2.5A10 5 0 01120 20z").expect("valid compact path");
    assert!(
        path.iter()
            .any(|segment| matches!(segment, PathSegment::CubicTo { .. }))
    );
}

#[test]
fn normalizes_path_length_dashes() {
    let image = decode(br#"<svg xmlns='http://www.w3.org/2000/svg'>
      <rect width='100' height='100' fill='none' stroke='black' stroke-dasharray='.25' pathLength='4'/>
      <path d='M0 0H100V100H0Z' fill='none' stroke='black' stroke-dasharray='.25' pathLength='4'/>
      <circle cx='100' cy='100' r='100' fill='none' stroke='black' stroke-dasharray='.25' pathLength='4'/>
    </svg>"#).expect("path length SVG");
    let dashes = image
        .commands()
        .iter()
        .filter_map(|command| match command {
            DrawCommand::Stroke { style, .. } => style.dash_array.first().copied(),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(dashes.len(), 3);
    assert!((dashes[0] - 25.0).abs() < 1e-9);
    assert!((dashes[0] - dashes[1]).abs() < 1e-9);
    assert!((dashes[2] - 39.27).abs() < 0.01);
}

#[test]
fn applies_svg_stacking_blending_and_isolation() {
    let image = decode(
        br#"<svg xmlns='http://www.w3.org/2000/svg'>
          <style>
            rect { mix-blend-mode: screen }
            #normal > rect { mix-blend-mode: normal }
          </style>
          <rect width='10' height='10' fill='red' style='z-index:1'/>
          <rect width='10' height='10' fill='lime'/>
          <g isolation='isolate'><rect width='10' height='10'/></g>
          <g id='normal'><rect width='10' height='10'/></g>
        </svg>"#,
    )
    .expect("stacking and compositing SVG");
    let fills = image
        .commands()
        .iter()
        .filter_map(|command| match command {
            DrawCommand::Fill { paint, .. } => Some(image.paint(*paint)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(matches!(fills[0], Paint::Solid(Color { green, .. }) if *green == 1.0));
    assert!(matches!(fills[3], Paint::Solid(Color { red, .. }) if *red == 1.0));
    assert_eq!(
        image
            .commands()
            .iter()
            .filter(|command| matches!(
                command,
                DrawCommand::PushScope {
                    blend_mode: crate::BlendMode::Screen,
                    ..
                }
            ))
            .count(),
        3
    );
    assert_eq!(
        image
            .commands()
            .iter()
            .filter(|command| matches!(command, DrawCommand::PushScope { isolated: true, .. }))
            .count(),
        1
    );
}

#[test]
fn supports_css_transforms_paint_order_and_keyword_case() {
    let image = decode(
        br#"<svg xmlns='http://www.w3.org/2000/svg'>
          <rect width='10' height='10' color='red' fill='CURRENTCOLOR' stroke='blue'
                style='transform:translate(29px,11px);paint-order:stroke fill'/>
        </svg>"#,
    )
    .expect("static CSS rendering properties");
    let commands = image
        .commands()
        .iter()
        .filter(|command| {
            matches!(
                command,
                DrawCommand::Fill { .. } | DrawCommand::Stroke { .. }
            )
        })
        .collect::<Vec<_>>();
    assert!(matches!(
        commands.as_slice(),
        [DrawCommand::Stroke { .. }, DrawCommand::Fill { .. }]
    ));
    assert!(commands.iter().all(|command| match command {
        DrawCommand::Fill { transform, .. } | DrawCommand::Stroke { transform, .. } => {
            transform.e == 29.0 && transform.f == 11.0
        }
        _ => false,
    }));
    assert!(image.paints().any(|(_, paint)| {
        matches!(paint, Paint::Solid(Color { red, green, blue, .. }) if *red == 1.0 && *green == 0.0 && *blue == 0.0)
    }));
}

#[test]
fn inherits_complete_paint_order_and_rejects_invalid_transform_units() {
    let image = decode(
        br##"<svg xmlns='http://www.w3.org/2000/svg'>
          <defs><marker id='dot' markerUnits='userSpaceOnUse' markerWidth='2' markerHeight='2'>
            <rect width='2' height='2'/>
          </marker></defs>
          <g paint-order='markers stroke'>
            <path d='M0 0L10 0Z' fill='red' stroke='blue' marker-start='url(#dot)'/>
          </g>
        </svg>"##,
    )
    .expect("inherited complete paint order");
    let layers = image
        .commands()
        .iter()
        .filter_map(|command| match command {
            DrawCommand::Fill { .. } => Some("fill"),
            DrawCommand::Stroke { .. } => Some("stroke"),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(layers, ["fill", "stroke", "fill"]);
    assert_eq!(
        decode(b"<svg><rect width='1' height='1' transform='scale(2px)'/></svg>"),
        Err(VectorDecodeError::InvalidData)
    );
}

#[test]
fn clips_marker_viewports_and_measures_general_cubic_paths() {
    let image = decode(
        br##"<svg xmlns='http://www.w3.org/2000/svg'>
          <defs><marker id='mark' markerUnits='userSpaceOnUse' markerWidth='2' markerHeight='2'>
            <rect x='-10' y='-10' width='20' height='20'/>
          </marker></defs>
          <path d='M0 0L10 0' marker-start='url(#mark)'/>
          <path d='M0 0C0 0 100 0 100 0C100 0 100 100 100 100C100 100 0 100 0 100C0 100 0 0 0 0Z'
                fill='none' stroke='black' stroke-dasharray='.25' pathLength='4'/>
        </svg>"##,
    )
    .expect("clipped marker and cubic path metrics");
    assert!(image.commands().iter().any(|command| {
        matches!(command, DrawCommand::PushScope { clips, .. } if !clips.is_empty())
    }));
    let dash = image.commands().iter().find_map(|command| match command {
        DrawCommand::Stroke { style, .. } if !style.dash_array.is_empty() => {
            style.dash_array.first().copied()
        }
        _ => None,
    });
    assert!(dash.is_some_and(|dash| (dash - 25.0).abs() < 1e-9));
}

#[test]
fn expands_local_markers_with_context_paint() {
    let image = decode(
        br##"<svg xmlns='http://www.w3.org/2000/svg'>
          <defs><marker id='arrow' markerUnits='userSpaceOnUse' orient='auto'
            markerWidth='4' markerHeight='4' refX='2' refY='2'>
            <path d='M0 0L4 2L0 4Z' fill='context-stroke'/>
          </marker></defs>
          <path d='M10 10L30 10' fill='none' stroke='lime' marker-end='url(#arrow)'/>
        </svg>"##,
    )
    .expect("bounded local marker");
    assert_eq!(
        image
            .commands()
            .iter()
            .filter(|command| matches!(command, DrawCommand::Fill { .. }))
            .count(),
        1
    );
    assert!(
        image.paints().any(|(_, paint)| {
            matches!(paint, Paint::Solid(Color { green, .. }) if *green == 1.0)
        })
    );
}

#[test]
fn recovers_valid_path_prefix_and_bounds_source_size() {
    let path = parse_path("M0 0L10 0L20 Z ignored").expect("valid path prefix");
    assert!(path.contains(&PathSegment::LineTo(Point { x: 10.0, y: 0.0 })));
    assert_eq!(
        decode(&vec![b' '; super::MAX_SOURCE_BYTES + 1]),
        Err(VectorDecodeError::ResourceLimit)
    );
}

#[test]
fn applies_gradient_color_interpolation() {
    let image = decode(
        br##"<svg xmlns='http://www.w3.org/2000/svg'>
          <defs><linearGradient id='g' color-interpolation='linearRGB'>
            <stop stop-color='black'/><stop offset='1' stop-color='white'/>
          </linearGradient></defs>
          <rect width='10' height='10' fill='url(#g)'/>
        </svg>"##,
    )
    .expect("linear RGB gradient");
    assert!(image.paints().any(|(_, paint)| {
        matches!(paint, Paint::LinearGradient { stops, .. } if stops.iter().all(|stop| stop.color.color_space == crate::VectorColorSpace::LinearSrgb))
    }));
}

#[test]
fn ignores_unsupported_content_and_keeps_supported_siblings() {
    for fragment in [
        "<?example ignored?>",
        "<script>ignored()</script>",
        "<text>ignored</text>",
        "<image href=\"https://example.invalid/image.png\"/>",
        "<filter id=\"blur\"/>",
        "<rect width=\"10\" height=\"10\" filter=\"url(#blur)\" marker-start=\"url(#missing)\"/>",
        "<circle cx=\"20\" cy=\"20\" r=\"1em\"/>",
    ] {
        let source = format!(
            "{prefix}<svg xmlns=\"http://www.w3.org/2000/svg\">{body}</svg>",
            prefix = if fragment.starts_with("<?") {
                fragment
            } else {
                ""
            },
            body = if fragment.starts_with("<?") {
                ""
            } else {
                fragment
            }
        );
        decode(source.as_bytes()).unwrap_or_else(|error| panic!("{fragment}: {error:?}"));
    }
    let image = decode(
        br#"<?example ignored?><svg xmlns="http://www.w3.org/2000/svg">
          <script>ignored()</script>
          <text>ignored</text>
          <image href="https://example.invalid/image.png"/>
          <filter id="blur"/>
          <rect width="10" height="10" filter="url(#blur)" marker-start="url(#missing)"/>
          <circle cx="20" cy="20" r="1em"/>
        </svg>"#,
    )
    .expect("unsupported features should be ignored");
    assert_eq!(
        image
            .commands()
            .iter()
            .filter(|command| matches!(command, DrawCommand::Fill { .. }))
            .count(),
        1
    );
}

#[test]
fn preserves_xml_structure_and_ignores_foreign_namespaces() {
    let image = decode(
        br#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:rdf="urn:rdf">
          <rdf:metadata><rdf:value>ignored</rdf:value></rdf:metadata>
          <g style="display:none"><rect width="5" height="5"/></g>
          <rect width="10" height="10"/>
        </svg>"#,
    )
    .expect("foreign metadata should not hide supported SVG content");
    assert_eq!(
        image
            .commands()
            .iter()
            .filter(|command| matches!(command, DrawCommand::Fill { .. }))
            .count(),
        1
    );
    assert_eq!(
        decode(b"outside<svg xmlns='http://www.w3.org/2000/svg'/>"),
        Err(VectorDecodeError::InvalidData)
    );
    assert_eq!(
        decode(b"<svg xmlns='http://www.w3.org/2000/svg'><bad:item/></svg>"),
        Err(VectorDecodeError::InvalidData)
    );
    assert_eq!(
            decode(br#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:a="urn:x" xmlns:b="urn:x" a:value="1" b:value="2"/>"#),
            Err(VectorDecodeError::InvalidData)
        );
    assert_eq!(
        decode(br#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xml="urn:wrong"/>"#),
        Err(VectorDecodeError::InvalidData)
    );
}

#[test]
fn decodes_shapes_transforms_styles_and_clips() {
    let image = decode(br#"<svg xmlns="http://www.w3.org/2000/svg" width="120" height="80">
            <defs><clipPath id="clip"><circle cx="20" cy="20" r="15"/></clipPath></defs>
            <g transform="translate(2, -3) rotate(10)" opacity=".5" clip-path="url(#clip)" color="blue">
              <rect x="1" y="2" width="30" height="20" rx="3" fill="currentColor"/>
              <ellipse cx="50" cy="20" rx="10" ry="5" fill="none" stroke="rgba(255,0,0,.5)" stroke-dasharray="2 3 4"/>
              <polygon points="0,0 10,0 5,10"/>
            </g></svg>"#).expect("supported SVG should decode");
    assert!(image.commands().iter().any(
        |command| matches!(command, DrawCommand::PushScope { clips, .. } if !clips.is_empty())
    ));
    assert!(image.commands().iter().any(|command| matches!(command, DrawCommand::Stroke { style, .. } if style.dash_array.len() == 6)));
}

#[test]
fn applies_transform_lists_in_svg_order() {
    let image = decode(
        br#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
          <g transform="translate(20 30) scale(2)">
            <rect width="10" height="5"/>
          </g>
        </svg>"#,
    )
    .expect("transformed SVG should decode");
    let bounds = image
        .commands()
        .iter()
        .find_map(|command| match command {
            DrawCommand::Fill { bounds, .. } => Some(*bounds),
            _ => None,
        })
        .expect("transformed rectangle");
    assert_eq!(bounds.x, 20.0);
    assert_eq!(bounds.y, 30.0);
    assert_eq!(bounds.width, 20.0);
    assert_eq!(bounds.height, 10.0);
}

#[test]
fn resolves_active_viewports_and_visibility() {
    let image = decode(
        br#"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100" viewBox="0 0 100 50" visibility="hidden">
          <rect width="50%" height="50%"/>
          <rect width="50%" height="50%" visibility="visible"/>
        </svg>"#,
    )
    .expect("viewport percentages should decode");
    let fills = image
        .commands()
        .iter()
        .filter_map(|command| match command {
            DrawCommand::Fill { bounds, .. } => Some(bounds),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(fills.len(), 1);
    assert_eq!(fills[0].width, 100.0);
    assert_eq!(fills[0].height, 50.0);
    assert!(matches!(
        image.commands().first(),
        Some(DrawCommand::PushScope { clips, .. }) if clips.len() == 1
    ));

    let nested = decode(
        br#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="50" viewBox="0 0 100 50">
          <svg x="10" y="5" width="50" height="20" viewBox="0 0 10 10">
            <rect width="100%" height="100%"/>
          </svg>
        </svg>"#,
    )
    .expect("nested viewport should decode");
    let bounds = nested
        .commands()
        .iter()
        .find_map(|command| match command {
            DrawCommand::Fill { bounds, .. } => Some(*bounds),
            _ => None,
        })
        .expect("nested rectangle");
    assert_eq!(bounds.x, 25.0);
    assert_eq!(bounds.y, 5.0);
    assert_eq!(bounds.width, 20.0);
    assert_eq!(bounds.height, 20.0);
}

#[test]
fn resolves_user_space_gradient_percentages() {
    let image = decode(
        br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 50">
          <defs><linearGradient id="g" gradientUnits="userSpaceOnUse" x2="100%"><stop/><stop offset="1"/></linearGradient></defs>
          <rect width="100" height="50" fill="url(#g)"/>
        </svg>"##,
    )
    .expect("user-space gradient percentages should decode");
    assert!(image.paints().any(|(_, paint)| {
        matches!(paint, Paint::LinearGradient { end, .. } if end.x == 100.0)
    }));
}

#[test]
fn disables_invalid_and_zero_sized_shapes() {
    for element in [
        "<circle r='-1'/>",
        "<ellipse rx='-1' ry='-1'/>",
        "<ellipse rx='1' ry='0'/>",
        "<rect width='-1' height='1'/>",
        "<rect width='1' height='0'/>",
    ] {
        let source = format!("<svg xmlns='http://www.w3.org/2000/svg'>{element}</svg>");
        let image =
            decode(source.as_bytes()).unwrap_or_else(|error| panic!("{element}: {error:?}"));
        assert!(!image.commands().iter().any(|command| matches!(
            command,
            DrawCommand::Fill { .. } | DrawCommand::Stroke { .. }
        )));
    }
    assert_eq!(
        decode(b"<svg><path d='M0 0L1 1' stroke='black' stroke-width='-1'/></svg>"),
        Err(VectorDecodeError::InvalidData)
    );

    let ellipse = decode(b"<svg><ellipse cx='10' cy='10' rx='-5' ry='5'/></svg>")
        .expect("negative ellipse radius computes to auto");
    assert!(
        ellipse
            .commands()
            .iter()
            .any(|command| matches!(command, DrawCommand::Fill { .. }))
    );
}

#[test]
fn validates_numeric_and_color_list_syntax() {
    let image = decode(
        br#"<svg xmlns="http://www.w3.org/2000/svg"><rect width="1" height="1" fill="RGB(100% 0% 0% / 50%)"/></svg>"#,
    )
    .expect("modern RGB syntax should decode");
    assert!(image.paints().any(|(_, paint)| {
        matches!(paint, Paint::Solid(Color { red, alpha, .. }) if *red == 1.0 && *alpha > 0.49 && *alpha < 0.51)
    }));

    for source in [
        "<svg viewBox='0,,0 1 1'/>",
        "<svg viewBox='0-1 1 1'/>",
        "<svg><rect width='1' height='1' transform='translate(1,)'/></svg>",
        "<svg><rect width='1' height='1' transform='translate(1),'/></svg>",
        "<svg><rect width='1' height='1' transform='translate(1)scale(2)'/></svg>",
        "<svg><path d='M0,,0'/></svg>",
        "<svg><rect width='1' height='1' fill='rgb(1,,2,3)'/></svg>",
        "<svg><rect width='1' height='1' fill='rgb(1,2 3)'/></svg>",
    ] {
        assert_eq!(
            decode(source.as_bytes()),
            Err(VectorDecodeError::InvalidData),
            "{source}"
        );
    }
}

#[test]
fn resolves_luminance_and_alpha_masks() {
    let image = decode(
        br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="80">
          <rect x="10" y="10" width="40" height="20" mask="url(#fade)"/>
          <defs>
            <mask id="fade" mask-type="alpha" maskContentUnits="objectBoundingBox">
              <rect width=".5" height="1" fill="white"/>
            </mask>
          </defs>
        </svg>"##,
    )
    .expect("local mask should decode");
    let mask = image
        .commands()
        .iter()
        .find_map(|command| match command {
            DrawCommand::PushScope {
                mask: Some(mask), ..
            } => Some(mask),
            _ => None,
        })
        .expect("masked scope");
    assert_eq!(mask.mask_type, MaskType::Alpha);
    assert!(!mask.commands.is_empty());
    assert_eq!(mask.region.bounds.x, 6.0);
    assert_eq!(mask.region.bounds.width, 48.0);
}

#[test]
fn rotated_masks_use_the_untransformed_object_bounding_box() {
    let image = decode(
        br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
          <defs><mask id="mask"><rect width="100" height="100" fill="white"/></mask></defs>
          <rect x="10" y="20" width="40" height="10" transform="rotate(45)" mask="url(#mask)"/>
        </svg>"##,
    )
    .expect("rotated masked shape");
    let mask = image
        .commands()
        .iter()
        .find_map(|command| match command {
            DrawCommand::PushScope {
                mask: Some(mask), ..
            } => Some(mask),
            _ => None,
        })
        .expect("mask scope");
    assert!(matches!(
        image.path(mask.region.path).first(),
        Some(PathSegment::MoveTo(Point { x, y })) if (*x - 6.0).abs() < 1e-9 && (*y - 19.0).abs() < 1e-9
    ));
}

#[test]
fn resolves_forward_nested_masks_and_rejects_cycles() {
    decode(br##"<svg xmlns="http://www.w3.org/2000/svg">
          <rect width="20" height="20" mask="url(#outer)"/>
          <defs>
            <mask id="inner"><rect width="20" height="20" fill="white"/></mask>
            <mask id="outer"><g mask="url(#inner)"><circle cx="10" cy="10" r="10" fill="white"/></g></mask>
          </defs>
        </svg>"##)
        .expect("nested forward masks should decode");
    assert_eq!(
        decode(
            br##"<svg xmlns="http://www.w3.org/2000/svg">
              <defs><mask id="cycle"><rect width="1" height="1" mask="url(#cycle)"/></mask></defs>
              <rect width="1" height="1" mask="url(#cycle)"/>
            </svg>"##
        ),
        Err(VectorDecodeError::InvalidData)
    );
}

#[test]
fn resolves_gradient_stops_and_forward_references() {
    let image = decode(br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10">
          <rect width="10" height="10" fill="url(#derived)"/>
          <defs>
            <linearGradient id="base"><stop offset="0" stop-color="#f00"/><stop offset="100%" stop-color="#00f8"/></linearGradient>
            <linearGradient id="derived" href="#base" x2="50%" spreadMethod="reflect"/>
          </defs>
        </svg>"##).expect("local gradient should resolve");
    assert!(image.paints().any(|(_, paint)| matches!(paint, Paint::LinearGradient { stops, spread: crate::SpreadMethod::Reflect, .. } if stops.len() == 2)));
}

#[test]
fn css_keeps_valid_fallback_when_later_value_is_invalid() {
    let image = decode(br#"<svg xmlns="http://www.w3.org/2000/svg"><rect width="1" height="1" style="fill:red;fill:context-fill"/></svg>"#).expect("invalid CSS declaration should be ignored");
    assert!(
        image
            .paints()
            .any(|(_, paint)| matches!(paint, Paint::Solid(Color { red, .. }) if *red == 1.0))
    );
}

#[test]
fn recognized_malformed_svg_never_becomes_unknown_input() {
    let bytes = b"<!-- leading comment --><svg><path d='M0 nan'/></svg>";
    assert!(is_svg(bytes));
    assert_eq!(decode(bytes), Err(VectorDecodeError::InvalidData));
}

#[test]
fn uses_css_absolute_units_and_normalized_percentages() {
    let image = decode(
        br#"<svg width="300" height="400"><circle cx="150" cy="200" r="10%" fill="none" stroke="black" stroke-width="1in" stroke-dasharray="1pc 12pt"/></svg>"#,
    )
    .expect("length units");
    let style = image
        .commands()
        .iter()
        .find_map(|command| match command {
            DrawCommand::Stroke { style, .. } => Some(style),
            _ => None,
        })
        .expect("stroke");
    assert_eq!(style.width, 96.0);
    assert_eq!(style.dash_array, [16.0, 16.0]);
}

#[test]
fn resolves_paint_fallbacks_and_gradient_templates() {
    let image = decode(br##"<svg width="100" height="100"><defs>
      <linearGradient id="base" gradientUnits="userSpaceOnUse" x2="75" gradientTransform="translate(2)" spreadMethod="reflect"><stop stop-color="red"/></linearGradient>
      <linearGradient id="derived" href="#base"/></defs>
      <rect width="10" height="10" fill="url(#missing) currentColor" color="blue"/>
      <rect x="20" width="10" height="10" fill="url(#derived)"/></svg>"##).expect("paint resolution");
    assert!(
        image
            .paints()
            .any(|(_, paint)| matches!(paint, Paint::Solid(Color { blue, .. }) if *blue == 1.0))
    );
    assert!(image.paints().any(|(_, paint)| matches!(paint, Paint::LinearGradient { end, spread: crate::SpreadMethod::Reflect, transform, .. } if end.x == 75.0 && transform.e == 2.0)));
}

#[test]
fn uses_fallback_for_singular_gradient_transforms() {
    let image = decode(br##"<svg width="20" height="10"><defs>
      <linearGradient id="linear" gradientTransform="scale(0)"><stop stop-color="red"/></linearGradient>
      <radialGradient id="radial" gradientTransform="matrix(0 0 0 0 0 0)"><stop stop-color="red"/></radialGradient>
    </defs><rect width="10" height="10" fill="url(#linear) green"/>
    <rect x="10" width="10" height="10" fill="url(#radial) green"/></svg>"##)
        .expect("singular gradient fallbacks");
    assert_eq!(
        image
            .paints()
            .filter(
                |(_, paint)| matches!(paint, Paint::Solid(Color { green, .. }) if *green > 0.49)
            )
            .count(),
        2
    );
}

#[test]
fn instantiates_symbol_viewports_without_rendering_definitions() {
    let image = decode(br##"<svg width="100" height="100"><symbol id="s" viewBox="0 0 10 10"><rect width="10" height="10"/></symbol><use href="#s" x="20" y="30" width="40" height="20"/></svg>"##).expect("symbol use");
    let fills = image
        .commands()
        .iter()
        .filter_map(|command| match command {
            DrawCommand::Fill { bounds, .. } => Some(*bounds),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(fills.len(), 1);
    assert_eq!(
        fills[0],
        crate::Rect {
            x: 30.0,
            y: 30.0,
            width: 20.0,
            height: 20.0
        }
    );
}

#[test]
fn applies_bounded_embedded_css_cascade() {
    let image = decode(br##"<svg width="10" height="10"><style><![CDATA[
      rect { fill: red; } .shape { fill: green; } #target.shape { fill: blue !important; }
      g rect { fill: white; } rect:hover { fill: white; }
    ]]></style><rect id="target" class="shape extra" width="10" height="10" style="fill: yellow"/></svg>"##).expect("embedded CSS");
    assert!(
        image
            .paints()
            .any(|(_, paint)| matches!(paint, Paint::Solid(Color { blue, .. }) if *blue == 1.0))
    );
}

#[test]
fn applies_css_geometry_to_shapes() {
    let image = decode(
        br#"<svg width="200" height="100"><style>
      circle { cx: 25%; cy: 50%; r: 10%; }
      ellipse { rx: auto; ry: 20px; }
    </style>
    <circle cx="1" cy="1" r="1"/>
    <ellipse cx="100" cy="50"/>
    <rect x="120" y="10" style="width: 40px; height: 30px"/>
    </svg>"#,
    )
    .expect("CSS geometry");
    let bounds = image
        .commands()
        .iter()
        .filter_map(|command| match command {
            DrawCommand::Fill { bounds, .. } => Some(*bounds),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(bounds.len(), 3);
    let radius = 200.0_f64.hypot(100.0) / std::f64::consts::SQRT_2 * 0.1;
    assert!((bounds[0].x - (50.0 - radius)).abs() < 1e-9);
    assert!((bounds[0].y - (50.0 - radius)).abs() < 1e-9);
    assert_eq!(bounds[1].width, 40.0);
    assert_eq!(bounds[1].height, 40.0);
    assert_eq!(bounds[2].x, 120.0);
    assert_eq!(bounds[2].y, 10.0);
    assert_eq!(bounds[2].width, 40.0);
    assert_eq!(bounds[2].height, 30.0);

    let cascade = decode(
        br#"<svg width="100" height="100"><style>
      rect { width: 10px; width: invalid; height: 20px !important; }
      #target { height: 30px; }
    </style><rect id="target" width="5" height="5" style="width: 40px"/></svg>"#,
    )
    .expect("geometry cascade");
    let bounds = cascade
        .commands()
        .iter()
        .find_map(|command| match command {
            DrawCommand::Fill { bounds, .. } => Some(*bounds),
            _ => None,
        })
        .expect("CSS-sized rectangle");
    assert_eq!(bounds.width, 40.0);
    assert_eq!(bounds.height, 20.0);
}

#[test]
fn uses_default_viewport_and_disables_zero_viewboxes() {
    let default = decode(b"<svg><rect width='1' height='1'/></svg>").expect("default size");
    assert_eq!(default.size().width, 300.0);
    assert_eq!(default.size().height, 150.0);

    let root = decode(
        b"<svg width='200' height='100' viewBox='0 0 0 10'><rect width='10' height='10'/></svg>",
    )
    .expect("zero root viewBox");
    assert!(root.commands().is_empty());

    let symbol = decode(
        br##"<svg width="200" height="100">
      <symbol id="empty" viewBox="0 0 10 0"><rect width="10" height="10"/></symbol>
      <use href="#empty" width="100" height="100"/>
    </svg>"##,
    )
    .expect("zero symbol viewBox");
    assert!(!symbol.commands().iter().any(|command| matches!(
        command,
        DrawCommand::Fill { .. } | DrawCommand::Stroke { .. }
    )));
    assert_eq!(
        decode(b"<svg viewBox='0 0 -1 10'/></svg>"),
        Err(VectorDecodeError::InvalidData)
    );
}
