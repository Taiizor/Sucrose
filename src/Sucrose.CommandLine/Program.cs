using E = Skylark.Exception;
using HA = Sucrose.CommandLine.Helper.Arguments;

namespace Sucrose.CommandLine
{
    internal class Program
    {
        internal static void Main(string[] Args)
        {
            try
            {
                HA.Parse(Args);
            }
            catch (E Ex)
            {
                Console.WriteLine(Ex.Message);
            }
        }
    }
}