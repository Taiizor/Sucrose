using SLHA = Sucrose.Localizer.Helper.Against;
using SLHC = Sucrose.Localizer.Helper.Creating;
using SLHCTX = Sucrose.Localizer.Helper.CsvToXaml;
using SLHXTC = Sucrose.Localizer.Helper.XamlToCsv;

namespace Sucrose.Localizer.Helper
{
    internal static class Program
    {
        public static void Start()
        {
            Console.WriteLine("Select the operation you want to start from below:");

            Console.WriteLine();

            Console.WriteLine("1. Convert XAML files to CSV files");
            Console.WriteLine("2. Convert CSV files to XAML files");
            Console.WriteLine("3. Check the CSV files against each other");
            Console.WriteLine("4. Create a new language file from CSV files");

            Console.WriteLine();

            Console.Write("Your selection: ");
            string selection = Console.ReadLine();

            string csvDirectory = string.Empty;
            string xamlDirectory = string.Empty;

            Console.WriteLine();

            switch (selection)
            {
                case "1":
                    Console.Write("Location of XAML files: ");
                    xamlDirectory = Console.ReadLine();

                    Console.Write("Location to save CSV files: ");
                    csvDirectory = Console.ReadLine();

                    Console.WriteLine();

                    SLHXTC.Convert(xamlDirectory, csvDirectory);
                    break;
                case "2":
                    Console.Write("Location of CSV files: ");
                    csvDirectory = Console.ReadLine();

                    Console.Write("Location to save XAML files: ");
                    xamlDirectory = Console.ReadLine();

                    Console.WriteLine();

                    SLHCTX.Convert(csvDirectory, xamlDirectory);
                    break;
                case "3":
                    Console.Write("Location of CSV files: ");
                    csvDirectory = Console.ReadLine();

                    Console.WriteLine();

                    SLHA.Check(csvDirectory);
                    break;
                case "4":
                    Console.Write("Location of CSV files: ");
                    csvDirectory = Console.ReadLine();

                    Console.WriteLine();

                    Console.Write("Source language (EN): ");
                    string sourceLang = Console.ReadLine();

                    Console.Write("Output language (TR): ");
                    string outputLang = Console.ReadLine();

                    Console.Write("Output language name (English): ");
                    string outputLangName = Console.ReadLine();

                    Console.WriteLine();

                    SLHC.Create(csvDirectory, sourceLang, outputLang, outputLangName);
                    break;
                default:
                    Console.WriteLine("Invalid selection.");

                    Console.WriteLine();
                    break;
            }
        }
    }
}