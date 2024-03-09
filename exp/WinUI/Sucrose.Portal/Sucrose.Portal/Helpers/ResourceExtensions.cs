using Microsoft.Windows.ApplicationModel.Resources;

namespace Sucrose.Portal.Helpers;

public static class ResourceExtensions
{
    private static readonly ResourceLoader _resourceLoader = new();

    public static string GetLocalized(this string resourceKey)
    {
        return _resourceLoader.GetString(resourceKey);
    }
}
