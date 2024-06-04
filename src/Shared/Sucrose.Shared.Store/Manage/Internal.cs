using System.Net.Http;
using SPSSS = Sucrose.Portal.Services.StoreService;

namespace Sucrose.Shared.Store.Manage
{
    internal static class Internal
    {
        public static bool State = true;

        public static readonly HttpClient Client = new()
        {
            Timeout = Timeout.InfiniteTimeSpan //TimeSpan.FromSeconds(30)
        };

        public static SPSSS StoreService { get; } = new();
    }
}