using Sucrose.Discord;

namespace Sucrose.Test
{
    internal class Program
    {
        static void Main(string[] args)
        {
            Hook DH = new();
            DH.Initialize();
            DH.SetPresence();

            Console.ReadKey();

            DH.Dispose();
        }
    }
}