export interface Term {
  text: string;
  reading?: string;
  defTags?: string[];
  rules?: string[];
  popularity?: number;
  definition: DictionaryDefinition;
  sequence?: number;
  tags?: string[];
}

export class TermEntry {
  static fromBank([
    text,
    reading,
    defTags,
    rules,
    popularity,
    definition,
    sequence,
    tags,
  ]: DictionaryTerm) {
    return new TermEntry({
      text,
      reading,
      defTags: defTags ? defTags.split(" ") : undefined,
      rules: rules ? rules.split(" ") : undefined,
      popularity,
      definition,
      sequence,
      tags: tags ? tags.split(" ") : undefined,
    });
  }

  constructor(public json: Term) {}
}

/**
 * term_bank_${number}.json
 *
 * The term bank for term information. This is where dictionary readings, definitions, and such are stored.
 *
 * @see https://github.com/themoeway/yomitan/tree/master/ext/data/schemas/dictionary-term-bank-v3-schema.json
 */
type DictionaryTermBankV3 = DictionaryTerm[];

type DictionaryTerm = [
  // "description": "Information about a single term.",
  // "minItems": 8,
  // "maxItems": 8,

  string,
  // "description": "The text for the term."
  // "description": "Reading of the term, or an empty string if the reading is the same as the term."
  string,
  string | null,
  // "description": "String of space-separated tags for the definition. An empty string is treated as no tags."
  string,
  // "description": "String of space-separated rule identifiers for the definition which is used to validate deinflection.
  // An empty string should be used for words which aren't inflected."
  number,
  // "description": "Score used to determine popularity.
  // Negative values are more rare and positive values are more frequent.
  // This score is also used to sort search results."
  DictionaryDefinition,
  number,
  // "type": "integer",
  // "description": "Sequence number for the term.
  // Terms with the same sequence number can be shown together when the \"resultOutputMode\" option is set to \"merge\"."
  string,
  // "description": "String of space-separated tags for the term. An empty string is treated as no tags."
];

type DictionaryDefinition =
  | string
  | DictionaryDefinitionObject
  | DictionaryDefinitionInflected;

type DictionaryDefinitionObject =
  | DictionaryDefinition_text
  | DictionaryDefinition_structured
  | DictionaryDefinition_img;

type DictionaryDefinitionInflected = [
  // "description": "Deinflection of the term to an uninflected term.",
  // "minItems": 2,
  // "maxItems": 2,

  string,
  // "description": "The uninflected term."
  string[],
  // "description": "A chain of inflection rules that produced the inflected term",
  // "description": "A single inflection rule."
];

interface DictionaryDefinition_text {
  type: "text";
  text: string;
  // "description": "Single definition for the term."
}

interface DictionaryDefinition_structured {
  type: "structured-content";
  content: StructuredContent;
}

interface DictionaryDefinition_img {
  type: "image";
  path: string;
  width?: number;
  // "type": "integer",
  // "description": "Preferred width of the image.",
  // "minimum": 1
  height?: number;
  // "type": "integer",
  // "description": "Preferred height of the image.",
  // "minimum": 1
  title?: string;
  // "description": "Hover text for the image."
  alt?: string;
  // "description": "Alt text for the image."
  description?: string;
  // "description": "Description of the image."
  pixelated?: boolean;
  // "description": "Whether or not the image should appear pixelated at sizes larger than the image's native resolution.",
  // "default": false
  imageRendering?: "auto" | "pixelated" | "crisp-edges";
  // "description": "Controls how the image is rendered. The value of this field supersedes the pixelated field.",
  // "default": "auto"
  appearance?: "auto" | "monochrome";
  // "description": "Controls the appearance of the image.
  // The \"monochrome\" value will mask the opaque parts of the image using the current text color.",
  // "default": "auto"
  background?: boolean;
  // "description": "Whether or not a background color is displayed behind the image.",
  // "default": true
  collapsed?: boolean;
  // "description": "Whether or not the image is collapsed by default.",
  // "default": false
  collapsible?: boolean;
  // "description": "Whether or not the image can be collapsed.",
  // "default": true
}

type StructuredContent =
  | string
  | StructuredContent[]
  | StructuredContent_br
  | StructuredContent_container
  | StructuredContent_table
  | StructuredContent_styled_container
  | StructuredContent_img
  | StructuredContent_link;

interface StructuredContent_br {
  tag: "br";
  data?: StructuredContentData;
}

interface StructuredContent_container {
  tag: "ruby" | "rt" | "rp" | "table" | "thead" | "tbody" | "tfoot" | "tr";
  content?: StructuredContent;
  data?: StructuredContentData;
  lang?: string;
}

interface StructuredContent_table {
  tag: "td" | "th";
  content?: StructuredContent;
  data?: StructuredContentData;
  colSpan?: number;
  // "type": "integer",
  // "minimum": 1
  rowSpan?: number;
  // "type": "integer",
  // "minimum": 1
  lang?: string;
}

interface StructuredContent_styled_container {
  tag: "span" | "div" | "ol" | "ul" | "li" | "details" | "summary";
  content?: StructuredContent;
  data?: StructuredContentData;
  style: StructuredContentStyle;
  title?: string;
  // "description": "Hover text for the element."
  open?: boolean;
  // "description": "Whether or not the details element is open by default."
  lang?: string;
}

interface StructuredContent_img {
  tag: "img";
  data?: StructuredContentData;
  path: string;
  // "description": "Path to the image file in the archive."
  width?: number;
  // "description": "Preferred width of the image.",
  // "minimum": 0
  height?: number;
  // "description": "Preferred height of the image.",
  // "minimum": 0
  title?: string;
  // "description": "Hover text for the image."
  alt?: string;
  // "description": "Alt text for the image."
  description?: string;
  // "description": "Description of the image."
  pixelated?: boolean;
  // "description": "Whether or not the image should appear pixelated at sizes larger than the image's native resolution.",
  // "default": false
  imageRendering?: "auto" | "pixelated" | "crisp-edges";
  // "description": "Controls how the image is rendered. The value of this field supersedes the pixelated field.",
  // "default": "auto"
  appearance?: "auto" | "monochrome";
  // "description": "Controls the appearance of the image. The \"monochrome\" value will mask the opaque parts of the image using the current text color.",
  // "default": "auto"
  background?: boolean;
  // "description": "Whether or not a background color is displayed behind the image.",
  // "default": true
  collapsed?: boolean;
  // "description": "Whether or not the image is collapsed by default.",
  // "default": false
  collapsible?: boolean;
  // "description": "Whether or not the image can be collapsed.",
  // "default": false
  verticalAlign:
    | "baseline"
    | "sub"
    | "super"
    | "text-top"
    | "text-bottom"
    | "middle"
    | "top"
    | "bottom";
  // "description": "The vertical alignment of the image.",
  border?: string;
  // "description": "Shorthand for border width, style, and color."
  borderRadius?: string;
  // "description": "Roundness of the corners of the image's outer border edge."
  sizeUnits: "px" | "em";
  // "description": "The units for the width and height.",
}

interface StructuredContent_link {
  tag: "a";
  content?: StructuredContent;
  href: string;
  // "description": "The URL for the link. URLs starting with a ? are treated as internal links to other dictionary content.",
  // "pattern": "^(?:https?:|\\?)[\\w\\W]*"
  lang?: string;
}

interface StructuredContentData {
  // "description": "Generic data attributes that should be added to the element.",
  [key: string]: string;
}

interface StructuredContentStyle {
  fontStyle?: "normal" | "italic";
  // "default": "normal"
  fontWeight?: "normal" | "bold";
  // "default": "normal"
  fontSize?: string;
  // "default": "medium"
  color?: string;
  background?: string;
  backgroundColor?: string;
  textDecorationLine?:
    | "none"
    | "underline"
    | "overline"
    | "line-through"
    | StructuredContentStyle_textDecorationLine[];
  // "default": "none"
  textDecorationStyle?: "solid" | "double" | "dotted" | "dashed" | "wavy";
  // "default": "solid"
  textDecorationColor?: string;
  borderColor?: string;
  borderStyle?: string;
  borderRadius?: string;
  borderWidth?: string;
  clipPath?: string;
  verticalAlign?:
    | "baseline"
    | "sub"
    | "super"
    | "text-top"
    | "text-bottom"
    | "middle"
    | "top"
    | "bottom";
  // "default": "baseline"
  textAlign?:
    | "start"
    | "end"
    | "left"
    | "right"
    | "center"
    | "justify"
    | "justify-all"
    | "match-parent";
  // "default": "start"
  textEmphasis?: string;
  textShadow?: string;
  margin?: string;
  marginTop?: number | string;
  // "default": 0
  marginLeft?: number | string;
  // "default": 0
  marginRight?: number | string;
  // "default": 0
  marginBottom?: number | string;
  // "default": 0
  padding?: string;
  paddingTop?: string;
  paddingLeft?: string;
  paddingRight?: string;
  paddingBottom?: string;
  wordBreak?: "normal" | "break-all" | "keep-all";
  // "default": "normal"
  whiteSpace?: string;
  // "default": "normal"
  cursor?: string;
  // "default": "auto"
  listStyleType?: string;
  // "default": "disc"
}

type StructuredContentStyle_textDecorationLine =
  | "underline"
  | "overline"
  | "line-through";
// "default": "none"
