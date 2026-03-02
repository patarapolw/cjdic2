import { Frequency } from "./kanji-meta";

export interface TermFreq {
  text: string;
  type: "freq";
  data: {
    reading?: string;
    freqNumber?: number;
    freqString?: string;
  };
}

export interface TermPitch {
  text: string;
  type: "pitch";
  data: {
    pitches: PitchEntry[];
  };
}

export interface TermIPA {
  text: string;
  type: "ipa";
  data: {
    reading: string;
    transcriptions: IPATranscription[];
  };
}

export type TermMeta = TermFreq | TermPitch | TermIPA;

export class TermMetaEntry {
  static fromBank([text, type, data]: DictionaryTermMeta) {
    switch (type) {
      case "freq":
        const freq = data;

        let reading = "";
        let freqNumber = 0;
        let freqString = "";

        if (freq) {
          if (typeof freq === "string") {
            freqString = freq;
          } else if (typeof freq === "number") {
            freqNumber = freq;
          } else if ("reading" in freq) {
            reading = freq.reading;

            const f = freq.frequency;
            if (typeof f === "string") {
              freqString = f;
            } else if (typeof f === "number") {
              freqNumber = f;
            } else {
              freqString = f.displayValue || "";
              freqNumber = f.value;
            }
          } else {
            freqString = freq.displayValue || "";
            freqNumber = freq.value;
          }
        }

        return new TermMetaEntry({
          text,
          type,
          data: {
            reading: reading || undefined,
            freqNumber: freqNumber || undefined,
            freqString: freqString || undefined,
          },
        });

      case "pitch":
        return new TermMetaEntry({ text, type, data });

      case "ipa":
        return new TermMetaEntry({ text, type, data });
    }

    return new TermMetaEntry({
      text,
      type,
      data: data as any,
    });
  }

  constructor(public json: TermMeta) {}
}

type DictionaryTermMetaBankV3 = DictionaryTermMeta[];

type DictionaryTermMeta =
  | DictionaryTermMeta_freq
  | DictionaryTermMeta_pitch
  | DictionaryTermMeta_ipa;

type DictionaryTermMeta_freq = [string, "freq", Frequency | ReadingFrequency];
type DictionaryTermMeta_pitch = [string, "pitch", Pitch];
type DictionaryTermMeta_ipa = [string, "ipa", IPA];

interface ReadingFrequency {
  reading: string;
  // "description": "Reading for the term."
  frequency: Frequency;
}

interface Pitch {
  reading: string;
  pitches: PitchEntry[];
}

interface PitchEntry {
  position: PitchPosition_integer | PitchPosition_string;
  nasal?: PitchNasal | PitchNasal[];
  devoice?: PitchDevoice | PitchDevoice[];
  tags?: string[];
  // "description": "Tag for this pitch accent. This typically corresponds to a certain type of part of speech."
}

type PitchPosition_integer = number;
// "type": "integer",
// "description": "Mora position of the pitch accent downstep. A value of 0 indicates that the word does not have a downstep (heiban).",
// "minimum": 0

type PitchPosition_string = string;
// "type": "string",
// "description": "Pitch level of each mora with H representing high and L representing low. For example: HHLL for a 4 mora word. Add an additional pitch level at the end to explicitly define the suffix.",
// "pattern": "^[HL]+$"

type PitchNasal = number;
// "type": "integer",
// "description": "Position of a mora with nasal sound.",
// "minimum": 0

type PitchDevoice = number;
// "type": "integer",
// "description": "Position of a mora with devoiced sound.",
// "minimum": 0

interface IPA {
  // "description": "IPA transcription information for the term.",
  reading: string;
  // "description": "Reading for the term."
  transcriptions: IPATranscription[];
}

interface IPATranscription {
  ipa?: string;
  // "description": "IPA transcription for the term."
  tags: string[];
  // "description": "Tag for this IPA transcription."
}
