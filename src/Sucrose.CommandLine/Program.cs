using SCLHA = Sucrose.CommandLine.Helper.Arguments;

namespace Sucrose.CommandLine
{
    internal class Program
    {
        internal static void Main(string[] Args)
        {
            SCLHA.Parse(Args);
        }
    }
}