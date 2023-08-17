using System.Net.Http;
using SPSSS = Sucrose.Portal.Services.StoreService;

namespace Sucrose.Shared.Store.Manage
{
    internal static class Internal
    {
        public static readonly HttpClient Client = new();

        public static SPSSS StoreService { get; } = new();

        public static readonly TimeSpan RequiredDuration = TimeSpan.FromDays(1);
    }
}