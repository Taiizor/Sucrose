using Microsoft.Extensions.DependencyInjection;
using System.Reflection;
using SPVMVM = Sucrose.Portal.ViewModels.ViewModel;

namespace Sucrose.Portal.Dependency
{
    internal static class Model
    {
        public static IServiceCollection AddTransientFromNamespace(this IServiceCollection services, string namespaceName, params Assembly[] assemblies)
        {
            foreach (Assembly assembly in assemblies)
            {
                IEnumerable<Type> types = assembly
                    .GetTypes()
                    .Where(type => type.IsClass && type.Namespace!.StartsWith(namespaceName, StringComparison.InvariantCultureIgnoreCase));

                foreach (Type? type in types)
                {
                    if (services.All(service => service.ServiceType != type))
                    {
                        if (type == typeof(SPVMVM))
                        {
                            continue;
                        }

                        _ = services.AddTransient(type);
                    }
                }
            }

            return services;
        }
    }
}