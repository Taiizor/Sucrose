using System.Windows;
using Application = System.Windows.Application;
using SMR = Sucrose.Memory.Readonly;

namespace Sucrose.Shared.Resources.Helper
{
    internal static class Resources
    {
        public static void SetLanguage(string Lang)
        {
            Lang = Lang.ToUpperInvariant();

            if (!CheckLanguage(Lang))
            {
                Lang = SMR.Language;
            }

            ResourceDictionary Resource = new()
            {
                Source = new Uri($"/Locales/Locale.{Lang}.xaml", UriKind.Relative)
            };

            RemoveResource();

            Application.Current.Resources.MergedDictionaries.Add(Resource);
        }

        private static bool CheckLanguage(string Lang)
        {
            try
            {
                return Application.LoadComponent(new Uri($"/Locales/Locale.{Lang}.xaml", UriKind.Relative)) is ResourceDictionary;
            }
            catch
            {
                return false;
            }
        }

        private static void RemoveResource()
        {
            List<ResourceDictionary> Resources = Application.Current.Resources.MergedDictionaries
                .Where(Resource => !string.IsNullOrEmpty(Resource.Source?.ToString()) && Resource.Source.ToString().Contains("/Locales/"))
                .ToList();

            foreach (ResourceDictionary Resource in Resources)
            {
                Application.Current.Resources.MergedDictionaries.Remove(Resource);
            }
        }
    }
}
