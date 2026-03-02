export interface KanjiMeta {
  character: string;
  type: string;
  freqNumber?: number;
  freqString?: string;
}

export class KanjiMetaEntry {
  static fromBank([character, type, freq]: DictionaryKanjiMeta) {
    let freqNumber = 0;
    let freqString = "";

    if (freq) {
      if (typeof freq === "string") {
        freqString = freq;
      } else if (typeof freq === "number") {
        freqNumber = freq;
      } else {
        freqString = freq.displayValue || "";
        freqNumber = freq.value;
      }
    }

    return new KanjiMetaEntry({
      character,
      type,
      freqNumber: freqNumber || undefined,
      freqString: freqString || undefined,
    });
  }

  constructor(public json: KanjiMeta) {}
}

/**
 * kanji_meta_bank_${number}.json
 *
 * The meta bank for kanji information. Right now, this is only used to store kanji frequency data.
 *
 * @see https://github.com/themoeway/yomitan/tree/master/ext/data/schemas/dictionary-kanji-meta-bank-v3-schema.json
 */
type DictionaryKanjiMetaBankV3 = DictionaryKanjiMeta[];

type DictionaryKanjiMeta = [
  // "description": "Metadata about a single kanji character.",
  // "minItems": 3,
  // "maxItems": 3,

  string,
  // "minLength": 1
  string,
  // "const": "freq",
  // "description": "Type of data. \"freq\" corresponds to frequency information."
  Frequency,
];

export type Frequency = string | number | FrequencyObject;

interface FrequencyObject {
  value: number;
  displayValue?: string;
}
