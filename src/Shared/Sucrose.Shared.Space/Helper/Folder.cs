using System.IO;

namespace Sucrose.Shared.Space.Helper
{
    internal static class Folder
    {
        public static void Create(string Source)
        {
            if (!Directory.Exists(Source))
            {
                string[] Locations = Source.Split(Path.DirectorySeparatorChar);

                string Current = Locations.First() + Path.DirectorySeparatorChar;

                foreach (string Location in Locations.Skip(1))
                {
                    Current = Path.Combine(Current, Location);

                    if (!Directory.Exists(Current))
                    {
                        Directory.CreateDirectory(Current);
                    }
                }
            }
        }
    }
}