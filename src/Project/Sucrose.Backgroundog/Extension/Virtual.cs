using System.Management;
using SSSHV = Sucrose.Shared.Space.Helper.Virtual;

namespace Sucrose.Backgroundog.Extension
{
    internal static class Virtual
    {
        public static bool VirtualityActive()
        {
            try
            {
                string Query = "SELECT * FROM Win32_Process WHERE ";

                foreach (string Name in SSSHV.GetApp())
                {
                    Query += $"Name = '{Name}' OR ";
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