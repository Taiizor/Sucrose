using CsvHelper;
using System.Globalization;

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
                using StreamReader reader = new(poeFilePath);
                using CsvReader csv = new(reader, CultureInfo.InvariantCulture);
                IEnumerable<dynamic> records = csv.GetRecords<dynamic>();

                Dictionary<string, List<string>> localizationData = new();

                string languageCode = Path.GetFileNameWithoutExtension(poeFilePath)?.Split('.').LastOrDefault();

                if (!localizationData.ContainsKey(languageCode))
                {
                    localizationData[languageCode] = new List<string>();
                }

                foreach (dynamic record in records)
                {
                    string term = record.Term;
                    string translate = record.Translate;
                    string context = record.Context;
                    string reference = record.Reference;
                    string comment = record.Comment;

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

                using (StreamWriter writer = new(outputFileName))
                using (CsvWriter csv = new(writer, CultureInfo.InvariantCulture))
                {
                    csv.WriteField("File");
                    csv.WriteField("Key");
                    csv.WriteField("Value");

                    foreach (string entry in kvp.Value)
                    {
                        csv.NextRecord();

                        string[] parts = entry.Split('⁞');

                        csv.WriteField(parts[0]);
                        csv.WriteField(parts[1]);
                        csv.WriteField(parts[2]);
                    }
                }

                Console.WriteLine($"Localization data for {languageCode} is written to {outputFileName}");
            }
        }
    }
}