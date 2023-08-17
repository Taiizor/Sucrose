using System.Net;
using System.Net.NetworkInformation;
using SMR = Sucrose.Memory.Readonly;

namespace Sucrose.Shared.Space.Helper
{
    internal static class Network
    {
        public static bool GetHostEntry()
        {
            try
            {
                _ = Dns.GetHostEntry(SMR.HostEntry);

                return true;
            }
            catch
            {
                return false;
            }
        }

        public static bool IsInternetAvailable()
        {
            return NetworkInterface.GetIsNetworkAvailable();
        }

        public static NetworkInterface[] AllNetworkInterfaces()
        {
            return NetworkInterface.GetAllNetworkInterfaces();
        }
    }
}