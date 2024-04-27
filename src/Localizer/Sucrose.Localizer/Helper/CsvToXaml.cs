using CsvHelper;
using System.Globalization;
using System.Text;
using SSECCE = Skylark.Standard.Extension.Cryptology.CryptologyExtension;

namespace Sucrose.Localizer.Helper
{
    internal static class CsvToXaml
    {
        public static void Convert(string csvDirectory, string xamlDirectory)
        {
            if (!Directory.Exists(xamlDirectory))
            {
                Directory.CreateDirectory(xamlDirectory);
            }

            Dictionary<string, StringBuilder> xamlContents = new();

            ProcessCsvFiles(xamlContents, csvDirectory, xamlDirectory);

            WriteXamlFiles(xamlContents);

            Console.WriteLine();
            Console.WriteLine("XAML file creation is complete.");
            Console.WriteLine();
        }

        private static void ProcessCsvFiles(Dictionary<string, StringBuilder> xamlContents, string csvDirectory, string xamlDirectory)
        {
            foreach (string csvFilePath in Directory.GetFiles(csvDirectory, "*.csv"))
            {
                using StreamReader reader = new(csvFilePath, Encoding.UTF8);
                using CsvReader csv = new(reader, CultureInfo.InvariantCulture);
                IEnumerable<dynamic> records = csv.GetRecords<dynamic>();

                foreach (dynamic record in records)
                {
                    string xamlFileName = record.File;
                    string key = record.Key;
                    string value = record.Value;

                    string xamlFilePath = Path.Combine(xamlDirectory, xamlFileName);

                    EnsureDirectoryExists(xamlFilePath);

                    if (!xamlContents.ContainsKey(xamlFilePath))
                    {
                        xamlContents[xamlFilePath] = new StringBuilder();
                        xamlContents[xamlFilePath].AppendLine("<ResourceDictionary\r\n    xmlns=\"http://schemas.microsoft.com/winfx/2006/xaml/presentation\"\r\n    xmlns:x=\"http://schemas.microsoft.com/winfx/2006/xaml\"\r\n    xmlns:system=\"clr-namespace:System;assembly=mscorlib\">");
                    }

                    AppendXamlContent(xamlContents[xamlFilePath], key, value);
                }
            }
        }

        private static void EnsureDirectoryExists(string filePath)
        {
            string directoryPath = Path.GetDirectoryName(filePath);
            if (!Directory.Exists(directoryPath))
            {
                Directory.CreateDirectory(directoryPath);
            }
        }

        private static void AppendXamlContent(StringBuilder xamlContent, string key, string value)
        {
            if (string.IsNullOrEmpty(key) && string.IsNullOrEmpty(value))
            {
                xamlContent.AppendLine();
            }
            else if (key == "Base64")
            {
                xamlContent.AppendLine(SSECCE.BaseToText(value));
            }
            else
            {
                xamlContent.AppendLine($"    <system:String x:Key=\"{key}\">{value}</system:String>");
            }
        }

        private static void WriteXamlFiles(Dictionary<string, StringBuilder> xamlContents)
        {
            foreach (KeyValuePair<string, StringBuilder> kvp in xamlContents)
            {
                string xamlFilePath = kvp.Key;

                kvp.Value.AppendLine("</ResourceDictionary>");

                using (StreamWriter writer = new(xamlFilePath, false, Encoding.UTF8))
                {
                    string value = kvp.Value.ToString();

                    if (!value.Contains("system:String x:Key"))
                    {
                        value = value.Replace("<ResourceDictionary\r\n    xmlns=\"http://schemas.microsoft.com/winfx/2006/xaml/presentation\"\r\n    xmlns:x=\"http://schemas.microsoft.com/winfx/2006/xaml\"\r\n    xmlns:system=\"clr-namespace:System;assembly=mscorlib\">", "﻿<ResourceDictionary xmlns=\"http://schemas.microsoft.com/winfx/2006/xaml/presentation\">");
                    }

                    writer.Write(value.Trim());
                }

                Console.WriteLine($"XAML file {xamlFilePath} is created.");
            }
        }
    }
}