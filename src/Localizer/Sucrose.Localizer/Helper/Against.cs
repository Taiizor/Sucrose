namespace Sucrose.Localizer.Helper
{
    internal static class Against
    {
        public static void Check(string csvDirectory)
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

        private static void CompareCsvFiles(string filePath1, string filePath2)
        {
            string[] lines1 = File.ReadAllLines(filePath1);
            string[] lines2 = File.ReadAllLines(filePath2);

            int minLineCount = Math.Min(lines1.Length, lines2.Length);

            for (int i = 0; i < minLineCount; i++)
            {
                string[] fields1 = lines1[i].Split(',');
                string[] fields2 = lines2[i].Split(',');

                string file1 = fields1[0];
                string file2 = fields2[0];

                string key1 = fields1[1];
                string key2 = fields2[1];

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
#if NET48_OR_GREATER
                string extension = filename.Substring(index);
                string nameWithoutExtension = filename.Substring(0, index);
#else
                string extension = filename[index..];
                string nameWithoutExtension = filename[..index];
#endif

                int lastIndex = nameWithoutExtension.LastIndexOf('.');

                if (lastIndex > 0)
                {
#if NET48_OR_GREATER
                    return nameWithoutExtension.Substring(0, lastIndex) + extension;
#else
                    return nameWithoutExtension[..lastIndex] + extension;
#endif
                }
            }

            return filename;
        }
    }
}