﻿using System.IO;

namespace Sucrose.Shared.Space.Helper
{
    internal static class Copy
    {
        public static void Folder(string Source, string Destination, bool Delete = true)
        {
            if (!Directory.Exists(Destination))
            {
                Directory.CreateDirectory(Destination);
            }

            if (Directory.Exists(Source))
            {
                foreach (string Record in Directory.GetFiles(Source))
                {
                    if (File.Exists(Record))
                    {
                        string DestinationFile = Path.Combine(Destination, Path.GetFileName(Record));

                        File.Copy(Record, DestinationFile, true);
                    }
                }

                foreach (string SubDirectory in Directory.GetDirectories(Source))
                {
                    if (Directory.Exists(SubDirectory))
                    {
                        string DestinationSubDirectory = Path.Combine(Destination, Path.GetFileName(SubDirectory));

                        Folder(SubDirectory, DestinationSubDirectory, Delete);
                    }
                }

                if (Delete)
                {
                    if (Directory.Exists(Source))
                    {
                        Directory.Delete(Source, true);
                    }
                }
            }
        }
    }
}