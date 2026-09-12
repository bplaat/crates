/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use super::path::parse_path;
use super::{decode, is_svg};
use crate::{Color, DrawCommand, MaskType, Paint, PathSegment, Point, VectorDecodeError};

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
fn rejects_negative_shape_and_stroke_dimensions() {
    for element in [
        "<circle r='-1'/>",
        "<ellipse rx='-1' ry='1'/>",
        "<rect width='1' height='1' rx='-1'/>",
        "<path d='M0 0L1 1' stroke='black' stroke-width='-1'/>",
    ] {
        let source = format!("<svg xmlns='http://www.w3.org/2000/svg'>{element}</svg>");
        assert_eq!(
            decode(source.as_bytes()),
            Err(VectorDecodeError::InvalidData),
            "{element}"
        );
    }
}

#[test]
fn validates_numeric_and_color_list_syntax() {
    let image = decode(
        br#"<svg xmlns="http://www.w3.org/2000/svg"><rect width="1" height="1" fill="rgb(100% 0% 0% / 50%)"/></svg>"#,
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
