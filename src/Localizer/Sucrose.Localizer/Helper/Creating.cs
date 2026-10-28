using CsvHelper;
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

            using (CsvWriter csvWriter = CsvWriter.ToFile(outputFile))
            using (CsvReader csvReader = CsvReader.FromFile(sourceFile))
            {
                csvWriter.WriteHeader("File", "Key", "Value");

                foreach (CsvDynamicRecord record in csvReader.GetRecords<CsvDynamicRecord>())
                {
                    string file = record["File"] ?? string.Empty;
                    string key = record["Key"] ?? string.Empty;
                    string value = record["Value"] ?? string.Empty;

                    file = file.Replace($".{sourceLang}.", $".{outputLang}.");

                    if (key == "Base64")
                    {
                        value = SSECCE.TextToBase(SSECCE.BaseToText(value).Replace($".{sourceLang}.", $".{outputLang}."));
                    }

                    csvWriter.WriteRecord(file, key, value);
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

            List<string[]> data = [];

            using (CsvReader csv = CsvReader.FromFile(localeFile))
            {
                foreach (CsvDynamicRecord record in csv.GetRecords<CsvDynamicRecord>())
                {
                    data.Add([record["File"] ?? string.Empty, record["Key"] ?? string.Empty, record["Value"] ?? string.Empty]);
                }
            }

            string[] newLanguageData = [$"{fileName}.xaml", $"{fileName}.{outputLang}", $"{outputLangName} (v0.1)"];

            data.Add(newLanguageData);

            data = data.OrderBy(d => d[1]).ToList();

            using CsvWriter csvWriter = CsvWriter.ToFile(localeFile);
            csvWriter.WriteHeader("File", "Key", "Value");

            foreach (string[] fields in data)
            {
                csvWriter.WriteRecord(fields);
            }
        }
    }
}