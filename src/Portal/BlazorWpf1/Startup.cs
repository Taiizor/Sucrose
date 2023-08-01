using BlazorWpf1.Data;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;

namespace BlazorWpf1
{
    public static class Startup
    {
        public static IServiceProvider? Services { get; private set; }

        public static void Init()
        {
            IHost host = Host.CreateDefaultBuilder()
                           .ConfigureServices(WireupServices)
                           .Build();
            Services = host.Services;
        }

        private static void WireupServices(IServiceCollection services)
        {
            services.AddWpfBlazorWebView();
            services.AddSingleton<WeatherForecastService>();

#if DEBUG
        services.AddBlazorWebViewDeveloperTools();
#endif
        }
    }
}