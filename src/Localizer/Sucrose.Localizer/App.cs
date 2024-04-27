using System.Globalization;
using System.Text;
using SHC = Skylark.Helper.Culture;
using SLHP = Sucrose.Localizer.Helper.Program;

namespace Sucrose.Localizer
{
    internal class App : IDisposable
    {
        public static void Main(string[] args)
        {
            try
            {
                Console.OutputEncoding = Encoding.UTF8;
                SHC.All = new CultureInfo(SHC.CurrentUITwoLetterISOLanguageName, true);

                SLHP.Start();
            }
            catch (Exception Exception)
            {
                Console.WriteLine();
                Console.WriteLine(Exception);
                Console.WriteLine();
            }
            finally
            {
                Console.Write("Press any key to exit..");

                Console.ReadKey();

                Close();
            }
        }

        public static void Close()
        {
            Environment.Exit(0);
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}