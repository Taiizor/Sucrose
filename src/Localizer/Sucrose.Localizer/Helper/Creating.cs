using CsvHelper;
using CsvHelper.Configuration;
using System.Globalization;
using SSECCE = Skylark.Standard.Extension.Cryptology.CryptologyExtension;

namespace Sucrose.Localizer.Helper
{
    internal static class Creating
    {
        public static void Create(string csvDirectory, string sourceLang, string outputLang, string outputLangName)
        {
            string sourceFile = Path.Combine(csvDirectory, $"{sourceLang}.csv");
            string outputFile = Path.Combine(csvDirectory, $"{outputLang}.csv");

            if (!File.Exists(sourceFile))
            {
                Console.WriteLine("The source CSV file does not exist.");
                Console.WriteLine();
                return;
            }

            if (File.Exists(outputFile))
            {
                Console.WriteLine("The output CSV file already exists.");
                Console.WriteLine();
                return;
            }

            using (StreamWriter writer = new(outputFile))
            using (CsvWriter csvWriter = new(writer, CultureInfo.InvariantCulture))
            using (StreamReader reader = new(sourceFile))
            using (CsvReader csvReader = new(reader, CultureInfo.InvariantCulture))
            {
                csvWriter.WriteField("File");
                csvWriter.WriteField("Key");
                csvWriter.WriteField("Value");

                IEnumerable<dynamic> records = csvReader.GetRecords<dynamic>();

                foreach (dynamic record in records)
                {
                    csvWriter.NextRecord();

                    csvWriter.WriteField(record.File.Replace($".{sourceLang}.", $".{outputLang}."));
                    csvWriter.WriteField(record.Key);

                    if (record.Key == "Base64")
                    {
                        record.Value = SSECCE.TextToBase(SSECCE.BaseToText(record.Value).Replace($".{sourceLang}.", $".{outputLang}."));
                    }

                    csvWriter.WriteField(record.Value);
                }
            }

            AddLocale(csvDirectory, outputLang, outputLangName);

            Console.WriteLine("CSV language file creating is complete.");
            Console.WriteLine();
        }

        private static void AddLocale(string csvDirectory, string outputLang, string outputLangName)
        {
            string fileName = "Locale";
            string localeFile = Path.Combine(csvDirectory, $"{fileName}.csv");

            List<string[]> data = new();

            using (StreamReader reader = new(localeFile))
            using (CsvReader csv = new(reader, new CsvConfiguration(CultureInfo.InvariantCulture)))
            {
                IEnumerable<dynamic> records = csv.GetRecords<dynamic>();

                foreach (dynamic record in records)
                {
                    string[] fields = new string[3];

                    fields[0] = record.File;
                    fields[1] = record.Key;
                    fields[2] = record.Value;

                    data.Add(fields);
                }
            }

            string[] newLanguageData = new string[] { $"{fileName}.xaml", $"{fileName}.{outputLang}", $"{outputLangName} (v0.1)" };

            data.Add(newLanguageData);

            data = data.OrderBy(d => d[1]).ToList();

            using StreamWriter writer = new(localeFile);
            using CsvWriter csvWriter = new(writer, CultureInfo.InvariantCulture);
            csvWriter.WriteField("File");
            csvWriter.WriteField("Key");
            csvWriter.WriteField("Value");

            foreach (string[] fields in data)
            {
                csvWriter.NextRecord();

                csvWriter.WriteField(fields[0]);
                csvWriter.WriteField(fields[1]);
                csvWriter.WriteField(fields[2]);
            }
        }
    }
}