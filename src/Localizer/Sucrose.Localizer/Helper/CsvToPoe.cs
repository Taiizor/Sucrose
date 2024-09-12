using CsvHelper;
using System.Globalization;

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

                using StreamReader reader = new(csvFilePath);
                using CsvReader csv = new(reader, CultureInfo.InvariantCulture);
                IEnumerable<dynamic> records = csv.GetRecords<dynamic>();

                Dictionary<string, List<string>> localizationData = new();

                string languageCode = Path.GetFileNameWithoutExtension(csvFilePath)?.Split('.').LastOrDefault();

                if (!localizationData.ContainsKey(languageCode))
                {
                    localizationData[languageCode] = new List<string>();
                }

                foreach (dynamic record in records)
                {
                    string file = record.File;
                    string key = string.IsNullOrEmpty(record.Key) ? $"PASS⦙{count1++}" : record.Key == "Base64" ? $"Base64⦙{count2++}" : record.Key;
                    string value = string.IsNullOrEmpty(record.Value) ? $"PASS⦙{count3++}" : record.Value;

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

                using (StreamWriter writer = new(outputFileName))
                using (CsvWriter poe = new(writer, CultureInfo.InvariantCulture))
                {
                    poe.WriteField("Term");
                    poe.WriteField("Translate");
                    poe.WriteField("Context");
                    poe.WriteField("Reference");
                    poe.WriteField("Comment");

                    foreach (string entry in kvp.Value)
                    {
                        poe.NextRecord();

                        string[] parts = entry.Split('⁞');

                        poe.WriteField(parts[0]);
                        poe.WriteField(parts[1]);
                        poe.WriteField(parts[2]);
                        poe.WriteField(parts[3]);
                        poe.WriteField(parts[4]);
                    }
                }

                Console.WriteLine($"Localization data for {languageCode} is written to {outputFileName}");
            }
        }
    }
}