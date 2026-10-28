using CsvHelper;
using SSECCE = Skylark.Standard.Extension.Cryptology.CryptologyExtension;

namespace Sucrose.Localizer.Helper
{
    internal static class PoeToCsv
    {
        public static void Convert(string poeDirectory, string csvDirectory)
        {
            if (!Directory.Exists(csvDirectory))
            {
                Directory.CreateDirectory(csvDirectory);
            }

            ProcessPoeFiles(poeDirectory, csvDirectory);

            Console.WriteLine();
            Console.WriteLine("POEditor file creation is complete.");
            Console.WriteLine();
        }

        private static void ProcessPoeFiles(string poeDirectory, string csvDirectory)
        {
            foreach (string poeFilePath in Directory.GetFiles(poeDirectory, "*.csv"))
            {
                using CsvReader csv = CsvReader.FromFile(poeFilePath);
                IEnumerable<CsvDynamicRecord> records = csv.GetRecords<CsvDynamicRecord>();

                Dictionary<string, List<string>> localizationData = [];

                string languageCode = Path.GetFileNameWithoutExtension(poeFilePath)?.Split('.').LastOrDefault();

                if (!localizationData.ContainsKey(languageCode))
                {
                    localizationData[languageCode] = [];
                }

                foreach (CsvDynamicRecord record in records)
                {
                    string term = record["Term"] ?? string.Empty;
                    string translate = record["Translate"] ?? string.Empty;
                    string context = record["Context"] ?? string.Empty;
                    string reference = record["Reference"] ?? string.Empty;
                    string comment = record["Comment"] ?? string.Empty;

                    if (term.Contains('⦙'))
                    {
                        term = term.Split('⦙')[0] == "PASS" ? string.Empty : term.Split('⦙')[0];
                    }

                    if (translate.Contains('⦙'))
                    {
                        translate = string.Empty;
                    }

                    localizationData[languageCode].Add($"{reference}⁞{term}⁞{translate}");
                }

                WriteLocalizationDataToCsv(localizationData, csvDirectory);
            }
        }

        private static void WriteLocalizationDataToCsv(Dictionary<string, List<string>> localizationData, string outputDirectory)
        {
            foreach (KeyValuePair<string, List<string>> kvp in localizationData)
            {
                string languageCode = kvp.Key;
                string outputFileName = Path.Combine(outputDirectory, $"{languageCode}.csv");

                using (CsvWriter csv = CsvWriter.ToFile(outputFileName))
                {
                    int count = 0;

                    csv.WriteHeader("Hash", "File", "Key", "Value");

                    foreach (string entry in kvp.Value)
                    {
                        string[] parts = entry.Split('⁞');

                        string hash = SSECCE.TextToSHA256($"{parts[0].Replace(languageCode, "")}.{(string.IsNullOrWhiteSpace(parts[1]) ? count++ : parts[1])}");

                        csv.WriteRecord(hash, parts[0], parts[1], parts[2]);
                    }
                }

                Console.WriteLine($"Localization data for {languageCode} is written to {outputFileName}");
            }
        }
    }
}