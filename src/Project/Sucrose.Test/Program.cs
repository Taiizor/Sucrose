using Sucrose.Discord;

namespace Sucrose.Test
{
    internal class Program
    {
        static void Main()
        {
            Hook DH = new();

            DH.Initialize();

            Console.ReadKey();

            DH.Dispose();
        }
    }
}