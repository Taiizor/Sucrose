using CsvHelper;
using Skylark.Standard.Extension.Cryptology;
using System.Globalization;
using System.Text.RegularExpressions;
using System.Xml;

namespace Sucrose.Localizer.Helper
{
    internal static class XamlToCsv
    {
        public static void Convert(string xamlDirectory, string csvDirectory)
        {
            if (!Directory.Exists(csvDirectory))
            {
                Directory.CreateDirectory(csvDirectory);
            }

            Dictionary<string, List<string>> localizationData = new();

            string[] xamlFiles = Directory.GetFiles(xamlDirectory, "*.xaml", SearchOption.AllDirectories);

            foreach (string file in xamlFiles)
            {
                string languageCode = Path.GetFileNameWithoutExtension(file)?.Split('.').LastOrDefault();

                if (languageCode != null)
                {
#if NET48_OR_GREATER
                    string relativeFilePath = file.Substring(xamlDirectory.Length + 1);
#else
                    string relativeFilePath = file[(xamlDirectory.Length + 1)..];
#endif

                    XmlDocument xmlDoc = new();
                    xmlDoc.Load(file);

                    string fileContent = File.ReadAllText(file);
                    string[] lines = fileContent.Split(new string[] { Environment.NewLine, "\n" }, StringSplitOptions.None);

                    if (!localizationData.ContainsKey(languageCode))
                    {
                        localizationData[languageCode] = new List<string>();
                    }

                    bool isInMergedDictionaries = false;
                    string startPattern = "    <ResourceDictionary.MergedDictionaries>";
                    string endPattern = "</ResourceDictionary.MergedDictionaries>";
                    string pattern = $"{Regex.Escape(startPattern)}.*?{Regex.Escape(endPattern)}";
                    MatchCollection mergedMatches = Regex.Matches(fileContent, pattern, RegexOptions.Singleline);

                    if (mergedMatches != null && mergedMatches.Count > 0)
                    {
                        isInMergedDictionaries = true;
                        AddMergedDictionaryEntry(localizationData, languageCode, relativeFilePath, mergedMatches);
                    }

                    XmlNodeList stringNodes = xmlDoc.GetElementsByTagName("system:String");
                    AddStringEntries(localizationData, languageCode, relativeFilePath, stringNodes, lines, isInMergedDictionaries);
                }
            }

            WriteLocalizationDataToCsv(localizationData, csvDirectory);

            Console.WriteLine();
            Console.WriteLine("Localization extraction is complete.");
            Console.WriteLine();
        }

        private static void AddMergedDictionaryEntry(Dictionary<string, List<string>> localizationData, string languageCode, string relativeFilePath, MatchCollection mergedMatches)
        {
            foreach (Match match in mergedMatches)
            {
                localizationData[languageCode].Add($"{relativeFilePath}⁞Base64⁞{CryptologyExtension.TextToBase(match.Value)}");
            }
        }

        private static void AddStringEntries(Dictionary<string, List<string>> localizationData, string languageCode, string relativeFilePath, XmlNodeList stringNodes, string[] lines, bool isInMergedDictionaries)
        {
            foreach (XmlNode node in stringNodes)
            {
                string value = node.InnerText;
                string key = node.Attributes?["x:Key"]?.Value;
                string entry = $"{relativeFilePath}⁞{key}⁞{value.Replace("&", "&amp;")}";

                if (isInMergedDictionaries)
                {
                    isInMergedDictionaries = false;
                    localizationData[languageCode].Add($"{relativeFilePath}⁞⁞");
                }

                localizationData[languageCode].Add(entry);

                for (int i = 0; i < lines.Length - 1; i++)
                {
                    if (lines[i].Contains($"x:Key=\"{key}\"") && string.IsNullOrEmpty(lines[i + 1]))
                    {
                        localizationData[languageCode].Add($"{relativeFilePath}⁞⁞");
                        break;
                    }
                }
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