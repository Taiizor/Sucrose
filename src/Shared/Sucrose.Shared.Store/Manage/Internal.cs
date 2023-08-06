using System.Net.Http;
using SSSID = Sucrose.Shared.Store.Interface.Data;

namespace Sucrose.Shared.Store.Manage
{
    internal static class Internal
    {
        public static readonly HttpClient Client = new();

        public static Dictionary<string, SSSID> Info = new();
    }
}