//! CSS properties catalog - Part 5: Modern Modules, Anchor Positioning & SVG (29 properties).

use super::meta::CssPropertyMeta;

pub const PROPS_MODERN: &[CssPropertyMeta] = &[
    // Group 9: Modern Modules, Anchor Positioning, View Transitions, SVG & Tables (29)
    CssPropertyMeta {
        name: "anchor-name",
        category: "Modern & SVG",
        syntax: "none | <dashed-ident>#",
        description: "Declares an element as a named anchor reference point for CSS Anchor Positioning.",
    },
    CssPropertyMeta {
        name: "position-anchor",
        category: "Modern & SVG",
        syntax: "auto | <dashed-ident>",
        description: "Binds an absolute/fixed popover element to an anchor target identified by anchor-name.",
    },
    CssPropertyMeta {
        name: "position-area",
        category: "Modern & SVG",
        syntax: "none | <position-area>",
        description: "Declarative grid-like positioning of anchored popovers around target (e.g. top center, bottom right).",
    },
    CssPropertyMeta {
        name: "position-try-options",
        category: "Modern & SVG",
        syntax: "none | <dashed-ident>#",
        description: "Fallback positioning strategies attempted in order when an anchored element would overflow viewport.",
    },
    CssPropertyMeta {
        name: "view-transition-name",
        category: "Modern & SVG",
        syntax: "none | auto | <custom-ident>",
        description: "Assigns view transition identity enabling cross-fade morphing animations during DOM tree updates.",
    },
    CssPropertyMeta {
        name: "object-fit",
        category: "Modern & SVG",
        syntax: "fill | contain | cover | none | scale-down",
        description: "Controls how images or video textures scale within their allocated bounds box.",
    },
    CssPropertyMeta {
        name: "object-position",
        category: "Modern & SVG",
        syntax: "<position>",
        description: "Specifies 2D alignment coordinates of media content within its element box.",
    },
    CssPropertyMeta {
        name: "container-type",
        category: "Modern & SVG",
        syntax: "normal | size | inline-size",
        description: "Establishes a container context for responsive @container layout queries.",
    },
    CssPropertyMeta {
        name: "container-name",
        category: "Modern & SVG",
        syntax: "none | <custom-ident>#",
        description: "Names container context so specific ancestor container dimensions can be queried.",
    },
    CssPropertyMeta {
        name: "contain-intrinsic-size",
        category: "Modern & SVG",
        syntax: "[ auto? [ <length> | none ] ]{1,2}",
        description: "Specifies placeholder dimensions for elements with content-visibility: auto to prevent layout scroll shifts.",
    },
    CssPropertyMeta {
        name: "fill",
        category: "Modern & SVG",
        syntax: "<paint> (none | <color> | <url>)",
        description: "Sets paint color, linear gradient, or radial gradient applied to interior of SVG vector shapes.",
    },
    CssPropertyMeta {
        name: "fill-rule",
        category: "Modern & SVG",
        syntax: "nonzero | evenodd",
        description: "Determines mathematical winding algorithm used to compute filled regions of self-intersecting vector paths.",
    },
    CssPropertyMeta {
        name: "fill-opacity",
        category: "Modern & SVG",
        syntax: "<number [0.0, 1.0]> | <percentage>",
        description: "Sets alpha transparency value for SVG path interior fill painting.",
    },
    CssPropertyMeta {
        name: "stroke",
        category: "Modern & SVG",
        syntax: "<paint> (none | <color> | <url>)",
        description: "Sets paint color or gradient used to outline SVG vector geometry lines and paths.",
    },
    CssPropertyMeta {
        name: "stroke-width",
        category: "Modern & SVG",
        syntax: "<length-percentage>",
        description: "Sets stroke line thickness for tessellated SVG vector paths.",
    },
    CssPropertyMeta {
        name: "stroke-linecap",
        category: "Modern & SVG",
        syntax: "butt | round | square",
        description: "Defines end cap geometry for open stroke subpaths.",
    },
    CssPropertyMeta {
        name: "stroke-linejoin",
        category: "Modern & SVG",
        syntax: "miter | round | bevel",
        description: "Defines join vertex shape where two line segments meet in an SVG stroke.",
    },
    CssPropertyMeta {
        name: "stroke-dasharray",
        category: "Modern & SVG",
        syntax: "none | <length-percentage>+#",
        description: "Specifies dash and gap pattern intervals for rendering dashed vector strokes.",
    },
    CssPropertyMeta {
        name: "table-layout",
        category: "Modern & SVG",
        syntax: "auto | fixed",
        description: "Selects table column width calculation algorithm (instant fixed vs content-dependent auto).",
    },
    CssPropertyMeta {
        name: "border-collapse",
        category: "Modern & SVG",
        syntax: "collapse | separate",
        description: "Controls whether adjacent table cell borders share a single edge or have distinct borders.",
    },
    CssPropertyMeta {
        name: "border-spacing",
        category: "Modern & SVG",
        syntax: "<length>{1,2}",
        description: "Sets horizontal and vertical distance separating adjacent table cell borders when separated.",
    },
    CssPropertyMeta {
        name: "caption-side",
        category: "Modern & SVG",
        syntax: "top | bottom",
        description: "Positions table caption header block either above or below table grid.",
    },
    CssPropertyMeta {
        name: "empty-cells",
        category: "Modern & SVG",
        syntax: "show | hide",
        description: "Controls whether borders and backgrounds are rendered for empty table cells.",
    },
    CssPropertyMeta {
        name: "columns",
        category: "Modern & SVG",
        syntax: "<'column-width'> || <'column-count'>",
        description: "Shorthand setting multi-column text column width and column count.",
    },
    CssPropertyMeta {
        name: "column-count",
        category: "Modern & SVG",
        syntax: "auto | <integer>",
        description: "Sets maximum number of newspaper-style text columns rendered in multi-column container.",
    },
    CssPropertyMeta {
        name: "column-width",
        category: "Modern & SVG",
        syntax: "auto | <length>",
        description: "Specifies ideal minimum width for columns in multi-column layout.",
    },
    CssPropertyMeta {
        name: "column-rule",
        category: "Modern & SVG",
        syntax: "<'column-rule-width'> || <'column-rule-style'> || <'column-rule-color'>",
        description: "Draws vertical divider line separating adjacent columns in multi-column text views.",
    },
    CssPropertyMeta {
        name: "break-inside",
        category: "Modern & SVG",
        syntax: "auto | avoid | avoid-page | avoid-column",
        description: "Prevents page, column, or region breaks inside cards or figures during printing.",
    },
    CssPropertyMeta {
        name: "break-before",
        category: "Modern & SVG",
        syntax: "auto | avoid | always | page | column",
        description: "Forces or avoids page or column break immediately preceding an element.",
    },
];
