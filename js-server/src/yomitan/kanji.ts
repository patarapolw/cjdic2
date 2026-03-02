export interface Kanji {
  character: string;
  onyomi?: string[];
  kunyomi?: string[];
  tags?: string[];
  meanings?: string[];
  stats?: Record<string, string>;
}

export class KanjiEntry {
  static fromBank([
    character,
    onyomi,
    kunyomi,
    tags,
    meanings,
    stats,
  ]: DictionaryKanji) {
    return new KanjiEntry({
      character,
      onyomi: onyomi ? onyomi.split(" ") : undefined,
      kunyomi: kunyomi ? kunyomi.split(" ") : undefined,
      tags: tags ? tags.split(" ") : undefined,
      meanings: meanings.length ? meanings : undefined,
      stats,
    });
  }

  constructor(public json: Kanji) {}
}

/**
 * kanji_bank_${number}.json
 *
 * Contains information used in the kanji viewer - meaning, readings, statistics, and codepoints.
 * Unfortunately a lot of the structuring is hardcoded and can't be customized nearly as much as with term definitions.
 *
 * @see https://github.com/yomidevs/yomitan/blob/master/ext/data/schemas/dictionary-kanji-bank-v3-schema.json
 */
type DictionaryKanjiBankV3 = DictionaryKanji[];

type DictionaryKanji = [
  // "description": "Information about a single kanji character.",
  // "minItems": 6,
  // "maxItems": 6,

  string,
  // "description": "Kanji character.",
  // "minLength": 1
  string,
  // "description": "String of space-separated onyomi readings for the kanji character. An empty string is treated as no readings."
  string,
  // "description": "String of space-separated kunyomi readings for the kanji character. An empty string is treated as no readings."
  string,
  // "description": "String of space-separated tags for the kanji character. An empty string is treated as no tags."
  string[],
  // "description": "Array of meanings for the kanji character.",

  {
    // "description": "Various stats for the kanji character.",
    [k: string]: string;
  },
];
