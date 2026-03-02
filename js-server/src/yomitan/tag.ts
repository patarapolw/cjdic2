export interface Tag {
  tag: string;
  category: string;
  sorting: number;
  notes: string;
  popularity: number;
}

export class TagEntry {
  static fromBank([tag, category, sorting, notes, popularity]: DictionaryTag) {
    return new TagEntry({
      tag,
      category,
      sorting,
      notes,
      popularity,
    });
  }

  constructor(public json: Tag) {}
}

/**
 * tag_bank_${number}.json
 *
 * The tag bank for term information.
 *
 * This is where you'll define tags for kanji and term dictionaries, like for example specifying parts of speech or kanken level.
 * These are generally displayed in Yomichan as grey tags next to the dictionary name.
 *
 * @see https://github.com/yomidevs/yomitan/blob/master/ext/data/schemas/dictionary-tag-bank-v3-schema.json
 */
type DictionaryTagBankV3 = DictionaryTag[];

/**
 * Information about a single tag.
 * "minItems": 5,
 * "maxItems": 5,
 */
type DictionaryTag = [
  /** Tag name. */
  string,
  /** Category for the tag. */
  string,
  /** Sorting order for the tag. */
  number,
  /** Notes for the tag. */
  string,
  /**
   * Score used to determine popularity.
   * Negative values are more rare and positive values are more frequent.
   * This score is also used to sort search results.
   */
  number,
];
