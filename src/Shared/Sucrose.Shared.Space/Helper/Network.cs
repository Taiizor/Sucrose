using System.Net.NetworkInformation;

namespace Sucrose.Shared.Space.Helper
{
    internal static class Network
    {
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