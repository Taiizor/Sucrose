using CsvHelper;

namespace Sucrose.Localizer.Helper
{
    internal static class CsvToPoe
    {
        public static void Convert(string csvDirectory, string poeDirectory)
        {
            if (!Directory.Exists(poeDirectory))
            {
                Directory.CreateDirectory(poeDirectory);
            }

            ProcessCsvFiles(csvDirectory, poeDirectory);

            Console.WriteLine();
            Console.WriteLine("POEditor file creation is complete.");
            Console.WriteLine();
        }

        private static void ProcessCsvFiles(string csvDirectory, string poeDirectory)
        {
            foreach (string csvFilePath in Directory.GetFiles(csvDirectory, "*.csv"))
            {
                int count1 = 0;
                int count2 = 0;
                int count3 = 0;

                using CsvReader csv = CsvReader.FromFile(csvFilePath);
                IEnumerable<CsvDynamicRecord> records = csv.GetRecords<CsvDynamicRecord>();

                Dictionary<string, List<string>> localizationData = [];

                string languageCode = Path.GetFileNameWithoutExtension(csvFilePath)?.Split('.').LastOrDefault();

                if (!localizationData.ContainsKey(languageCode))
                {
                    localizationData[languageCode] = [];
                }

                foreach (CsvDynamicRecord record in records)
                {
                    string file = record["File"] ?? string.Empty;
                    string rawKey = record["Key"] ?? string.Empty;
                    string rawValue = record["Value"] ?? string.Empty;

                    string key = string.IsNullOrEmpty(rawKey) ? $"PASS⦙{count1++}" : rawKey == "Base64" ? $"Base64⦙{count2++}" : rawKey;
                    string value = string.IsNullOrEmpty(rawValue) ? $"PASS⦙{count3++}" : rawValue;

                    if (key.Contains('⦙') || value.Contains('⦙'))
                    {
                        localizationData[languageCode].Add($"{key}⁞{value}⁞Don't Touch⁞{file}⁞Please don't touch the translation in this line.");
                    }
                    else
                    {
                        localizationData[languageCode].Add($"{key}⁞{value}⁞{string.Empty}⁞{file}⁞{string.Empty}");
                    }
                }

                WriteLocalizationDataToPoe(localizationData, poeDirectory);
            }
        }

        private static void WriteLocalizationDataToPoe(Dictionary<string, List<string>> localizationData, string outputDirectory)
        {
            foreach (KeyValuePair<string, List<string>> kvp in localizationData)
            {
                string languageCode = kvp.Key;
                string outputFileName = Path.Combine(outputDirectory, $"{languageCode}.csv");

                using (CsvWriter poe = CsvWriter.ToFile(outputFileName))
                {
                    poe.WriteHeader("Term", "Translate", "Context", "Reference", "Comment");

                    foreach (string entry in kvp.Value)
                    {
                        string[] parts = entry.Split('⁞');
                        poe.WriteRecord(parts[0], parts[1], parts[2], parts[3], parts[4]);
                    }
                }

                Console.WriteLine($"Localization data for {languageCode} is written to {outputFileName}");
            }
        }
    }
}