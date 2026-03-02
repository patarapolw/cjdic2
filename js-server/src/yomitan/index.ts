import "dotenv/config";

import AdmZip from "adm-zip";
import { readdirSync } from "fs";
import { MongoClient, ObjectId } from "mongodb";
import { join as joinPath } from "path";

import { Kanji, KanjiEntry } from "./kanji";
import { KanjiMeta, KanjiMetaEntry } from "./kanji-meta";
import { Tag, TagEntry } from "./tag";
import { Term, TermEntry } from "./term";
import { TermMeta, TermMetaEntry } from "./term-meta";

if (require.main === module) {
  const client = new MongoClient(process.env.MONGO_URI!);

  (async () => {
    await client.connect();

    const db = client.db("yomitan");

    const colMeta = db.collection<DictionaryIndex>("meta");
    await colMeta.createIndex({ title: 1 }, { unique: true });

    const colKanjiMeta = db.collection<KanjiMeta & { meta_id: ObjectId }>(
      "kanji-meta",
    );
    await colKanjiMeta.createIndex({ meta_id: 1 });

    const colKanji = db.collection<Kanji & { meta_id: ObjectId }>("kanji");
    await colKanji.createIndex({ meta_id: 1 });

    const colTag = db.collection<Tag & { meta_id: ObjectId }>("tag");
    await colTag.createIndex({ meta_id: 1 });

    const colTermMeta = db.collection<TermMeta & { meta_id: ObjectId }>(
      "term-meta",
    );
    await colTermMeta.createIndex({ meta_id: 1 });

    const colTerm = db.collection<Term & { meta_id: ObjectId }>("term");
    await colTerm.createIndex({ meta_id: 1 });

    const zipPath = "../src-tauri/resources/yomitan/ja";

    for (const f of readdirSync(zipPath)) {
      if (!f.endsWith(".zip")) continue;

      const zipFile = new AdmZip(joinPath(zipPath, f));
      const idx = zipFile.getEntry("index.json");
      if (!idx) continue;

      const meta: DictionaryIndex = JSON.parse(idx.getData().toString("utf-8"));
      const title = meta.title.replace(/\[.+?\]$/, "").trimEnd();
      if (title !== meta.title) {
        console.log(meta.title);
        meta.title = title;
      }
      console.log(meta.title);

      const it = await colMeta.insertOne(meta).catch((e) => {
        console.error(e);
        return null;
      });
      if (!it) continue;
      const meta_id = it.insertedId;

      for (const et of zipFile.getEntries()) {
        const f = et.entryName;

        if (f.startsWith("kanji_meta_bank")) {
          // yet empty
          await colKanjiMeta.insertMany(
            JSON.parse(et.getData().toString("utf-8")).map((t: any) => ({
              meta_id,
              ...KanjiMetaEntry.fromBank(t).json,
            })),
          );
        } else if (f.startsWith("kanji_bank")) {
          // yet empty
          await colKanji.insertMany(
            JSON.parse(et.getData().toString("utf-8")).map((t: any) => ({
              meta_id,
              ...KanjiEntry.fromBank(t).json,
            })),
          );
        } else if (f.startsWith("tag_bank")) {
          // 5 so far
          await colTag.insertMany(
            JSON.parse(et.getData().toString("utf-8")).map((t: any) => ({
              meta_id,
              ...TagEntry.fromBank(t).json,
            })),
          );
        } else if (f.startsWith("term_bank")) {
          // lots
          await colTerm.insertMany(
            JSON.parse(et.getData().toString("utf-8")).map((t: any) => ({
              meta_id,
              ...TermEntry.fromBank(t).json,
            })),
          );
        } else if (f.startsWith("term_meta_bank")) {
          // yet empty
          await colTermMeta.insertMany(
            JSON.parse(et.getData().toString("utf-8")).map((t: any) => ({
              meta_id,
              ...TermMetaEntry.fromBank(t).json,
            })),
          );
        }
      }
    }
  })().finally(async () => {
    await client.close();
  });
}

/**
 * The schema for the index.json file that contains metadata about the dictionary.
 * PLEASE ALWAYS PUT AS MUCH DETAIL IN THIS AS POSSIBLE.
 *
 * Note that this information can be displayed in Yomichan by going to the dictionaries overview page
 * and clicking the three dots, then Details....
 *
 * "isoLanguageCode": {
 *   "type": "string",
 *   "description": "ISO language code (ISO 639-1 where possible, ISO 639-3 otherwise).",
 *   "pattern": "^[a-z]{2,3}$"
 * }
 * "required": [
 *   "title",
 *   "revision"
 * ],
 * "anyOf": [
 *   {
 *     "required": ["format"]
 *   },
 *   {
 *     "required": ["version"]
 *   }
 * ],
 * "dependencies": {
 *   "isUpdatable": ["indexUrl", "downloadUrl"]
 * }
 *
 * @see https://github.com/yomidevs/yomitan/blob/master/ext/data/schemas/dictionary-index-schema.json
 */
interface DictionaryIndex {
  /** Title of the dictionary. */
  title: string;
  /** Revision of the dictionary. This value is displayed, and used to check for dictionary updates. */
  revision: string;
  /** Minimum version of Yomitan that is compatible with this dictionary. */
  minimumYomitanVersion?: string;
  /**
   * Whether or not this dictionary contains sequencing information for related terms.
   * @default false
   */
  sequenced?: boolean;
  /** Format of data found in the JSON data files. */
  format?: 1 | 2 | 3;
  /** Alias for format. */
  version?: 1 | 2 | 3;
  /** Creator of the dictionary. */
  author?: string;
  /**
   * Whether this dictionary contains links to its latest version.
   * @constant true
   */
  isUpdatable?: boolean;
  /** URL for the index file of the latest revision of the dictionary, used to check for updates. */
  indexUrl?: string;
  /** URL for the download of the latest revision of the dictionary. */
  downloadUrl?: string;
  /** URL for the source of the dictionary, displayed in the dictionary details. */
  url?: string;
  /** Description of the dictionary data. */
  description?: string;
  /** Attribution information for the dictionary data. */
  attribution?: string;
  /**
   * Language of the terms in the dictionary.
   * @ref #/definitions/isoLanguageCode
   */
  sourceLanguage?: string;
  /**
   * Main language of the definitions in the dictionary.
   * @ref #/definitions/isoLanguageCode
   */
  targetLanguage?: string;

  frequencyMode?: "occurrence-based" | "rank-based";
  /**
   * Tag information for terms and kanji. This object is obsolete and individual tag files should be used instead.
   * Information about a single tag. The object key is the name of the tag.
   */
  tagMeta?: {
    /** Category for the tag. */
    category?: string;
    /** Sorting order for the tag. */
    order?: number;
    /** Notes for the tag. */
    notes?: string;
    /** Score used to determine popularity.
     * Negative values are more rare and positive values are more frequent.
     * This score is also used to sort search results.
     */
    score?: number;
  };
}
