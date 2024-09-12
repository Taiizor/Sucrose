using SLHA = Sucrose.Localizer.Helper.Against;
using SLHC = Sucrose.Localizer.Helper.Creating;
using SLHCTP = Sucrose.Localizer.Helper.CsvToPoe;
using SLHCTX = Sucrose.Localizer.Helper.CsvToXaml;
using SLHPTC = Sucrose.Localizer.Helper.PoeToCsv;
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
            Console.WriteLine("3. Convert CSV files to POEditor files");
            Console.WriteLine("4. Convert POEditor files to CSV files");
            Console.WriteLine("5. Check the CSV files against each other");
            Console.WriteLine("6. Check the POEditor files against each other");
            Console.WriteLine("7. Create a new language file from CSV files");

            Console.WriteLine();

            Console.Write("Your selection: ");
            string selection = Console.ReadLine();

            string csvDirectory = string.Empty;
            string poeDirectory = string.Empty;
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

                    Console.Write("Location to save POEditor files: ");
                    poeDirectory = Console.ReadLine();

                    Console.WriteLine();

                    SLHCTP.Convert(csvDirectory, poeDirectory);
                    break;
                case "4":
                    Console.Write("Location of POEditor files: ");
                    poeDirectory = Console.ReadLine();

                    Console.Write("Location to save CSV files: ");
                    csvDirectory = Console.ReadLine();

                    Console.WriteLine();

                    SLHPTC.Convert(poeDirectory, csvDirectory);
                    break;
                case "5":
                    Console.Write("Location of CSV files: ");
                    csvDirectory = Console.ReadLine();

                    Console.WriteLine();

                    SLHA.CheckCsv(csvDirectory);
                    break;
                case "6":
                    Console.Write("Location of POEditor files: ");
                    poeDirectory = Console.ReadLine();

                    Console.WriteLine();

                    SLHA.CheckPoe(poeDirectory);
                    break;
                case "7":
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