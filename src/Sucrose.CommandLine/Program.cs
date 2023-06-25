using HA = Sucrose.CommandLine.Helper.Arguments;

namespace Sucrose.CommandLine
{
    internal class Program
    {
        internal static void Main(string[] Args)
        {
            HA.Parse(Args);
        }
    }
}