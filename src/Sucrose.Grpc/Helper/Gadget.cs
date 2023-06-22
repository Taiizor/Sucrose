using System.Net.Sockets;
using System.Net;

namespace Sucrose.Grpc.Helper
{
    internal static class Gadget
    {
        public static int AvailablePort(IPAddress Host)
        {
            TcpListener Listener = new(Host, 0);

            Listener.Start();

            int Port = ((IPEndPoint)Listener.LocalEndpoint).Port;

            Listener.Stop();

            return Port;
        }
    }
}