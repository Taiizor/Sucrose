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
                default:
                    Console.WriteLine("Invalid selection.");

                    Console.WriteLine();
                    break;
            }
        }
    }
}