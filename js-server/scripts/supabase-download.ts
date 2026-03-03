import "dotenv/config";

import { createClient } from "@supabase/supabase-js";

const supabaseUrl = process.env.VITE_SUPABASE_URL!;
const supabasePublishableKey = process.env.VITE_SUPABASE_PUBLISHABLE_KEY!;

const supabase = createClient(supabaseUrl, supabasePublishableKey);

async function main() {
  const { data, error } = await supabase.storage.from("yomitan").list("ja");
  if (error) throw error;
  if (data) {
    console.log(data.map((d) => d.id));

    const toDownload = data.filter(() => true);

    console.log(
      toDownload.map((d) =>
        supabase.storage.from("yomitan").getPublicUrl(`ja/${d.name}`),
      ),
    );
  }
}

if (require.main === module) {
  main();
}
