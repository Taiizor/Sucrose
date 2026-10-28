using CsvHelper;
using System.Text.RegularExpressions;

namespace Sucrose.Localizer.Helper
{
    internal static class Against
    {
        public static void ReindexCsv(string csvDirectory, string reindexerLang)
        {
            string reindexerSource = Path.Combine(csvDirectory, $"{reindexerLang}.csv");

            if (!File.Exists(reindexerSource))
            {
                Console.WriteLine($"Error: {reindexerLang}.csv file not found.");
                return;
            }

            Dictionary<int, string> reindexerLines = FileReadWithLines(reindexerSource);

            List<string> csvFiles = Directory.GetFiles(csvDirectory, "*.csv")
                .Where(filePath => Path.GetFileNameWithoutExtension(filePath).Length == 2)
                .ToList();

            csvFiles.Remove(reindexerSource);

            foreach (string csvFile in csvFiles)
            {
                Console.WriteLine($"-- Reindexing {Path.GetFileName(csvFile)} with {Path.GetFileName(reindexerSource)} --");

                Dictionary<int, string> newLines = [];
                Dictionary<int, string> csvLines = FileReadWithLines(csvFile);

                foreach (KeyValuePair<int, string> pair in csvLines)
                {
                    string[] fields = pair.Value.Split(',');

                    string hash = fields[0];
                    bool found = false;

                    foreach (KeyValuePair<int, string> reindexerPair in reindexerLines)
                    {
                        string[] reindexerFields = reindexerPair.Value.Split(',');

                        string reindexerHash = reindexerFields[0];

                        if (hash == reindexerHash)
                        {
                            if (reindexerPair.Key != pair.Key)
                            {
                                Console.WriteLine($"Success: Hash {hash} found in but not in the same line in reindexer file.");
                            }

                            newLines.Add(reindexerPair.Key, pair.Value);

                            found = true;

                            break;
                        }
                    }

                    if (!found)
                    {
                        Console.WriteLine($"Warning: Hash {hash} not found in reindexer file.");
                    }
                }

                Dictionary<int, string> orderedLines = newLines.OrderBy(kv => kv.Key).ToDictionary(kv => kv.Key, kv => kv.Value);

                File.WriteAllLines(csvFile, orderedLines.Values);

                Console.WriteLine();
            }

            Console.WriteLine();
            Console.WriteLine("CSV file Reindexing is complete.");
            Console.WriteLine();
        }

        public static void CheckCsv(string csvDirectory)
        {
            string[] csvFiles = Directory.GetFiles(csvDirectory, "*.csv")
                .Where(filePath => Path.GetFileNameWithoutExtension(filePath).Length == 2)
                .ToArray();

            for (int i = 0; i < csvFiles.Length; i++)
            {
                for (int j = i + 1; j < csvFiles.Length; j++)
                {
                    Console.WriteLine($"-- Comparing {Path.GetFileName(csvFiles[i])} and {Path.GetFileName(csvFiles[j])} --");
                    CompareCsvFiles(csvFiles[i], csvFiles[j]);
                    Console.WriteLine();
                }
            }

            Console.WriteLine();
            Console.WriteLine("CSV file checking is complete.");
            Console.WriteLine();
        }

        public static void CheckPoe(string poeDirectory)
        {
            string[] poeFiles = Directory.GetFiles(poeDirectory, "*.csv")
                .Where(filePath => Path.GetFileNameWithoutExtension(filePath).Length == 2)
                .ToArray();

            for (int i = 0; i < poeFiles.Length; i++)
            {
                for (int j = i + 1; j < poeFiles.Length; j++)
                {
                    Console.WriteLine($"-- Comparing {Path.GetFileName(poeFiles[i])} and {Path.GetFileName(poeFiles[j])} --");
                    ComparePoeFiles(poeFiles[i], poeFiles[j]);
                    Console.WriteLine();
                }
            }

            Console.WriteLine();
            Console.WriteLine("POEditor file checking is complete.");
            Console.WriteLine();
        }

        public static void AlphabeticIndexer(string csvDirectory, string languageCode)
        {
            string csvFilePath = Path.Combine(csvDirectory, $"{languageCode}.csv");

            if (!File.Exists(csvFilePath))
            {
                Console.WriteLine($"Error: {languageCode}.csv file not found.");
                return;
            }

            Console.WriteLine($"Processing alphabetic indexer for {languageCode}.csv...");

            List<CsvRecord> records = ReadCsvRecords(csvFilePath);
            List<CsvRecord> processedRecords = ProcessRecordsForSorting(records);

            // Write the processed records back to file
            WriteCsvRecords(csvFilePath, processedRecords);

            Console.WriteLine();
            Console.WriteLine($"Alphabetic indexing for {languageCode}.csv is complete.");
            Console.WriteLine();
        }

        private static List<CsvRecord> ReadCsvRecords(string filePath)
        {
            List<CsvRecord> records = [];

            using CsvReader csv = CsvReader.FromFile(filePath);

            while (csv.Read())
            {
                records.Add(new CsvRecord
                {
                    Hash = csv.GetString("Hash"),
                    File = csv.GetString("File"),
                    Key = csv.GetString("Key"),
                    Value = csv.GetString("Value")
                });
            }

            return records;
        }

        private static List<CsvRecord> ProcessRecordsForSorting(List<CsvRecord> records)
        {
            List<CsvRecord> processedRecords = [.. records];
            List<SortGroup> groups = IdentifySortGroups(processedRecords);

            Console.WriteLine($"Found {groups.Count} groups to process.");
            Console.WriteLine();

            // Sort each group
            foreach (SortGroup group in groups)
            {
                if (group.Records.Count > 1)
                {
                    string fileName = group.Records.FirstOrDefault()?.File ?? "Unknown";
                    Console.WriteLine($"-- Processing group: {Path.GetFileName(fileName)} (Start: {group.StartIndex + 1}, Records: {group.Records.Count}) --");

                    // Separate records that should not be sorted (Base64, empty keys)
                    List<CsvRecord> excludedRecords = group.Records
                        .Where(r => string.IsNullOrWhiteSpace(r.Key) ||
                                   r.Key.Equals("Base64", StringComparison.OrdinalIgnoreCase))
                        .ToList();

                    // Sort only the records that should be sorted
                    List<CsvRecord> sortableRecords = group.Records
                        .Where(r => !string.IsNullOrWhiteSpace(r.Key) &&
                                   !r.Key.Equals("Base64", StringComparison.OrdinalIgnoreCase))
                        .OrderBy(r => r.Key.Length)           // 1. Shortest to longest
                        .ThenBy(r => GetKeyForSorting(r.Key)) // 2. Alphabetical A-Z + numerical order
                        .ToList();

                    Console.WriteLine($"   Excluded: {excludedRecords.Count}, Sorted: {sortableRecords.Count}");

                    if (sortableRecords.Count > 0)
                    {
                        Console.WriteLine($"   First key: {sortableRecords.First().Key}");
                        Console.WriteLine($"   Last key: {sortableRecords.Last().Key}");
                    }

                    // Combine excluded records first, then sorted records
                    List<CsvRecord> sortedGroup = [.. excludedRecords, .. sortableRecords];

                    // Replace the group records in processed records
                    for (int i = 0; i < sortedGroup.Count && group.StartIndex + i < processedRecords.Count; i++)
                    {
                        processedRecords[group.StartIndex + i] = sortedGroup[i];
                    }

                    Console.WriteLine($"   Success: Group sorted and updated.");
                    Console.WriteLine();
                }
                else
                {
                    string fileName = group.Records.FirstOrDefault()?.File ?? "Unknown";
                    Console.WriteLine($"-- Skipping group: {Path.GetFileName(fileName)} (Single record) --");
                    Console.WriteLine();
                }
            }

            return processedRecords;
        }

        private static List<SortGroup> IdentifySortGroups(List<CsvRecord> records)
        {
            List<SortGroup> groups = [];
            int currentGroupStart = -1;
            string currentFile = null;

            for (int i = 0; i < records.Count; i++)
            {
                bool isEmptyOrSpecialKey = string.IsNullOrWhiteSpace(records[i].Key) ||
                                          string.IsNullOrWhiteSpace(records[i].Value) ||
                                          records[i].Key.Equals("Base64", StringComparison.OrdinalIgnoreCase);

                bool isFileChange = currentFile != null && records[i].File != currentFile;

                // Start a new group when we find the first non-empty record of a file
                if (!isEmptyOrSpecialKey && (currentGroupStart == -1 || isFileChange))
                {
                    // Finish previous group if exists
                    if (currentGroupStart != -1)
                    {
                        groups.Add(new SortGroup
                        {
                            StartIndex = currentGroupStart,
                            Records = records.Skip(currentGroupStart).Take(i - currentGroupStart).ToList()
                        });
                    }

                    currentGroupStart = i;
                    currentFile = records[i].File;
                }
                // End current group when we hit empty line or file change
                else if (isEmptyOrSpecialKey && currentGroupStart != -1)
                {
                    groups.Add(new SortGroup
                    {
                        StartIndex = currentGroupStart,
                        Records = records.Skip(currentGroupStart).Take(i - currentGroupStart).ToList()
                    });
                    currentGroupStart = -1;
                    currentFile = null;
                }
            }

            // Handle the last group if it doesn't end with empty line
            if (currentGroupStart != -1)
            {
                groups.Add(new SortGroup
                {
                    StartIndex = currentGroupStart,
                    Records = records.Skip(currentGroupStart).ToList()
                });
            }

            return groups;
        }

        private class SortGroup
        {
            public int StartIndex { get; set; }
            public List<CsvRecord> Records { get; set; } = [];
        }

        private static void WriteCsvRecords(string filePath, List<CsvRecord> records)
        {
            using CsvWriter csv = CsvWriter.ToFile(filePath);

            csv.WriteHeader("Hash", "File", "Key", "Value");

            foreach (CsvRecord record in records)
            {
                csv.WriteRecord(record.Hash, record.File, record.Key, record.Value);
            }
        }

        private class CsvRecord
        {
            public string Hash { get; set; } = string.Empty;
            public string File { get; set; } = string.Empty;
            public string Key { get; set; } = string.Empty;
            public string Value { get; set; } = string.Empty;
        }

        private static void CompareCsvFiles(string filePath1, string filePath2)
        {
            string[] lines1 = File.ReadAllLines(filePath1);
            string[] lines2 = File.ReadAllLines(filePath2);

            int minLineCount = Math.Min(lines1.Length, lines2.Length);

            for (int i = 0; i < minLineCount; i++)
            {
                string[] fields1 = lines1[i].Split(',');
                string[] fields2 = lines2[i].Split(',');

                string file1 = fields1[1];
                string file2 = fields2[1];

                string key1 = fields1[2];
                string key2 = fields2[2];

                bool areInSameRow = GetFilenameWithoutLanguageCode(file1) == GetFilenameWithoutLanguageCode(file2) && key1 == key2;

                if (areInSameRow)
                {
                    //Console.WriteLine($"Row {i + 1}: Present in both files.");
                }
                else
                {
                    Console.WriteLine($"Row {i + 1}: Present in both files but different.");
                }
            }

            if (lines1.Length != lines2.Length)
            {
                Console.WriteLine("Warning: Files are of different lengths!");
            }
        }

        private static void ComparePoeFiles(string filePath1, string filePath2)
        {
            string[] lines1 = File.ReadAllLines(filePath1);
            string[] lines2 = File.ReadAllLines(filePath2);

            int minLineCount = Math.Min(lines1.Length, lines2.Length);

            for (int i = 0; i < minLineCount; i++)
            {
                string[] fields1 = Regex.Replace(lines1[i], @"""(.*?)""", m => m.Value.Replace(",", "")).Split(',');
                string[] fields2 = Regex.Replace(lines2[i], @"""(.*?)""", m => m.Value.Replace(",", "")).Split(',');

                string file1 = fields1[3];
                string file2 = fields2[3];

                string key1 = fields1[0];
                string key2 = fields2[0];

                bool areInSameRow = GetFilenameWithoutLanguageCode(file1) == GetFilenameWithoutLanguageCode(file2) && key1 == key2;

                if (areInSameRow)
                {
                    //Console.WriteLine($"Row {i + 1}: Present in both files.");
                }
                else
                {
                    Console.WriteLine($"Row {i + 1}: Present in both files but different.");
                }
            }

            if (lines1.Length != lines2.Length)
            {
                Console.WriteLine("Warning: Files are of different lengths!");
            }
        }

        private static string GetFilenameWithoutLanguageCode(string filename)
        {
            int index = filename.LastIndexOf('.');

            if (index > 0)
            {
                string extension = filename[index..];
                string nameWithoutExtension = filename[..index];

                int lastIndex = nameWithoutExtension.LastIndexOf('.');

                if (lastIndex > 0)
                {
                    return nameWithoutExtension[..lastIndex] + extension;
                }
            }

            return filename;
        }

        private static string GetKeyForSorting(string key)
        {
            // Extract text and number parts for natural sorting
            Match match = Regex.Match(key, @"^(.+?)(\d+)$");

            if (match.Success)
            {
                string textPart = match.Groups[1].Value;
                int numberPart = int.Parse(match.Groups[2].Value);

                // Pad number with leading zeros for proper sorting
                return $"{textPart}{numberPart:D10}";
            }

            return key;
        }

        private static Dictionary<int, string> FileReadWithLines(string filePath)
        {
            Dictionary<int, string> lines = [];

            try
            {
                using StreamReader sr = new(filePath);
                int lineNumber = 1;
                string line;

                while ((line = sr.ReadLine()) != null)
                {
                    lines.Add(lineNumber, line);
                    lineNumber++;
                }
            }
            catch (FileNotFoundException)
            {
                Console.WriteLine("File not found.");
            }
            catch (IOException e)
            {
                Console.WriteLine("File reading error: " + e.Message);
            }

            return lines;
        }
    }
}