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
      reading: reading === text ? undefined : reading || undefined,
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
 * @see https://github.com/yomidevs/yomitan/tree/master/ext/data/schemas/dictionary-term-bank-v3-schema.json
 */
type DictionaryTermBankV3 = DictionaryTerm[];

/**
 * Information about a single term.
 * "minItems": 8,
 * "maxItems": 8,
 */
type DictionaryTerm = [
  /** The text for the term. */
  string,
  /** Reading of the term, or an empty string if the reading is the same as the term. */
  string,
  (
    /** String of space-separated tags for the definition. An empty string is treated as no tags. */
    string | null
  ),
  /**
   * String of space-separated rule identifiers for the definition which is used to validate deinflection.
   * An empty string should be used for words which aren't inflected.
   */
  string,
  /**
   * Score used to determine popularity.
   * Negative values are more rare and positive values are more frequent.
   * This score is also used to sort search results.
   */
  number,

  DictionaryDefinition,
  /**
   * Sequence number for the term.
   * Terms with the same sequence number can be shown together when the "resultOutputMode" option is set to "merge".
   *
   * "type": "integer",
   */
  number,
  /** String of space-separated tags for the term. An empty string is treated as no tags. */
  string,
];

type DictionaryDefinition =
  | string
  | DictionaryDefinitionObject
  | DictionaryDefinitionInflected;

type DictionaryDefinitionObject =
  | DictionaryDefinition_text
  | DictionaryDefinition_structured
  | DictionaryDefinition_img;

/**
 * Deinflection of the term to an uninflected term.
 *
 * "minItems": 2,
 * "maxItems": 2,
 */
type DictionaryDefinitionInflected = [
  /** The uninflected term. */
  string,
  /**
   * A chain of inflection rules that produced the inflected term.
   * Array of single inflection rules.
   */
  string[],
];

interface DictionaryDefinition_text {
  type: "text";
  /** Single definition for the term. */
  text: string;
}

interface DictionaryDefinition_structured {
  type: "structured-content";
  content: StructuredContent;
}

interface DictionaryDefinition_img {
  type: "image";
  path: string;
  /**
   * "type": "integer",
   * "description": "Preferred width of the image.",
   * "minimum": 1
   */
  width?: number;
  /**
   * "type": "integer",
   * "description": "Preferred height of the image.",
   * "minimum": 1
   */
  height?: number;
  /** Hover text for the image. */
  title?: string;
  /** Alt text for the image. */
  alt?: string;
  /** Description of the image. */
  description?: string;
  /**
   * Whether or not the image should appear pixelated at sizes larger than the image's native resolution.
   * @default false
   */
  pixelated?: boolean;
  /**
   * Controls how the image is rendered. The value of this field supersedes the pixelated field.
   * @default "auto"
   */
  imageRendering?: "auto" | "pixelated" | "crisp-edges";
  /**
   * Controls the appearance of the image.
   * The "monochrome" value will mask the opaque parts of the image using the current text color.
   * @default "auto"
   */
  appearance?: "auto" | "monochrome";
  /**
   * Whether or not a background color is displayed behind the image.
   * @default true
   */
  background?: boolean;
  /**
   * Whether or not the image is collapsed by default.
   * @default false
   */
  collapsed?: boolean;
  /**
   * Whether or not the image can be collapsed.
   * @default true
   */
  collapsible?: boolean;
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
  /**
   * "type": "integer",
   * "minimum": 1
   */
  colSpan?: number;
  /**
   * "type": "integer",
   * "minimum": 1
   */
  rowSpan?: number;
  lang?: string;
}

interface StructuredContent_styled_container {
  tag: "span" | "div" | "ol" | "ul" | "li" | "details" | "summary";
  content?: StructuredContent;
  data?: StructuredContentData;
  style: StructuredContentStyle;
  /** Hover text for the element. */
  title?: string;
  /** Whether or not the details element is open by default. */
  open?: boolean;
  lang?: string;
}

interface StructuredContent_img {
  tag: "img";
  data?: StructuredContentData;
  /** Path to the image file in the archive. */
  path: string;
  /**
   * Preferred width of the image.
   * "minimum": 0
   */
  width?: number;
  /**
   * Preferred height of the image.
   * "minimum": 0
   */
  height?: number;
  /** Hover text for the image. */
  title?: string;
  /** Alt text for the image. */
  alt?: string;
  /** Description of the image. */
  description?: string;
  /**
   * Whether or not the image should appear pixelated at sizes larger than the image's native resolution.
   * @default false
   */
  pixelated?: boolean;
  /**
   * Controls how the image is rendered. The value of this field supersedes the pixelated field.
   * @default "auto"
   */
  imageRendering?: "auto" | "pixelated" | "crisp-edges";
  /**
   * Controls the appearance of the image. The "monochrome" value will mask the opaque parts of the image using the current text color.
   * @default "auto"
   */
  appearance?: "auto" | "monochrome";
  /**
   * Whether or not a background color is displayed behind the image.
   * @default true
   */
  background?: boolean;
  /**
   * Whether or not the image is collapsed by default.
   * @default false
   */
  collapsed?: boolean;
  /**
   * Whether or not the image can be collapsed.
   * @default false
   */
  collapsible?: boolean;
  /** The vertical alignment of the image. */
  verticalAlign:
    | "baseline"
    | "sub"
    | "super"
    | "text-top"
    | "text-bottom"
    | "middle"
    | "top"
    | "bottom";
  /** Shorthand for border width, style, and color. */
  border?: string;
  /** Roundness of the corners of the image's outer border edge. */
  borderRadius?: string;
  /** The units for the width and height. */
  sizeUnits: "px" | "em";
}

interface StructuredContent_link {
  tag: "a";
  content?: StructuredContent;
  /**
   * The URL for the link. URLs starting with a ? are treated as internal links to other dictionary content.
   * "pattern": "^(?:https?:|\\?)[\\w\\W]*"
   */
  href: string;
  lang?: string;
}

/** Generic data attributes that should be added to the element. */
interface StructuredContentData {
  [key: string]: string;
}

interface StructuredContentStyle {
  /**
   * @default "normal"
   */
  fontStyle?: "normal" | "italic";
  /**
   * @default "normal"
   */
  fontWeight?: "normal" | "bold";
  /**
   * @default "medium"
   */
  fontSize?: string;
  color?: string;
  background?: string;
  backgroundColor?: string;
  /**
   * @default "none"
   */
  textDecorationLine?:
    | "none"
    | "underline"
    | "overline"
    | "line-through"
    | StructuredContentStyle_textDecorationLine[];
  /**
   * @default "solid"
   */
  textDecorationStyle?: "solid" | "double" | "dotted" | "dashed" | "wavy";
  textDecorationColor?: string;
  borderColor?: string;
  borderStyle?: string;
  borderRadius?: string;
  borderWidth?: string;
  clipPath?: string;
  /**
   * @default "baseline"
   */
  verticalAlign?:
    | "baseline"
    | "sub"
    | "super"
    | "text-top"
    | "text-bottom"
    | "middle"
    | "top"
    | "bottom";
  /**
   * @default "start"
   */
  textAlign?:
    | "start"
    | "end"
    | "left"
    | "right"
    | "center"
    | "justify"
    | "justify-all"
    | "match-parent";
  textEmphasis?: string;
  textShadow?: string;
  margin?: string;
  /**
   * @default 0
   */
  marginTop?: number | string;
  /**
   * @default 0
   */
  marginLeft?: number | string;
  /**
   * @default 0
   */
  marginRight?: number | string;
  /**
   * @default 0
   */
  marginBottom?: number | string;
  /**
   * @default 0
   */
  padding?: string;
  paddingTop?: string;
  paddingLeft?: string;
  paddingRight?: string;
  paddingBottom?: string;
  /**
   * @default "normal"
   */
  wordBreak?: "normal" | "break-all" | "keep-all";
  /**
   * @default "normal"
   */
  whiteSpace?: string;
  /**
   * @default "auto"
   */
  cursor?: string;
  /**
   * @default "disc"
   */
  listStyleType?: string;
}

/**
 * @default "none"
 */
type StructuredContentStyle_textDecorationLine =
  | "underline"
  | "overline"
  | "line-through";
