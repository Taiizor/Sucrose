using System.Management;

namespace Sucrose.Shared.Space.Helper
{
    internal static class Graphic
    {
        public static string[] AllVideoController()
        {
            List<string> Names = new();

            ManagementObjectSearcher Searcher = new("SELECT * FROM Win32_VideoController");

            foreach (ManagementObject Object in Searcher.Get().Cast<ManagementObject>())
            {
                Names.Add(Object["Name"].ToString().TrimStart().TrimEnd());
            }

            return Names.ToArray();
        }
    }
}