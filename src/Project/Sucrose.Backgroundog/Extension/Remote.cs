using System.Management;
using SSSHR = Sucrose.Shared.Space.Helper.Remote;

namespace Sucrose.Backgroundog.Extension
{
    internal static class Remote
    {
        public static bool DesktopActive()
        {
            try
            {
                string Query = "SELECT * FROM Win32_Process WHERE ";

                foreach (string appName in SSSHR.GetApp())
                {
                    Query += $"Name = '{appName}' OR ";
                }

                Query = Query.TrimEnd(" OR ".ToCharArray());

                ManagementObjectSearcher Searcher = new(Query);

                ManagementObjectCollection Collection = Searcher.Get();

                return Collection.Count > 0;
            }
            catch
            {
                return false;
            }
        }
    }
}