using SCHA = Sucrose.Commandog.Helper.Arguments;

namespace Sucrose.Commandog
{
    internal class App
    {
        internal static void Main(string[] Args)
        {
            SCHA.Parse(Args);
        }
    }
}