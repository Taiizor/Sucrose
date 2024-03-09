using System.Reflection;
using CommunityToolkit.Mvvm.ComponentModel;
using Microsoft.UI.Xaml.Controls;
using Rhythm.Contracts.Services;
using Rhythm.ViewModels;
using Rhythm.Views;

namespace Rhythm.Services;

public class PageService : IPageService
{
    private readonly Dictionary<string, Type> _pages = new();
    private readonly Dictionary<string, Type> _hiddenPages = new();
    private readonly Dictionary<string, string> _pageNames = new();

    public PageService()
    {
        Configure<MainViewModel, MainPage>();
        Configure<ContentGridViewModel, ContentGridPage>();
        Configure<ContentGridDetailViewModel, ContentGridDetailPage>();
        Configure<ListDetailsViewModel, ListDetailsPage>();
        Configure<DataGridViewModel, DataGridPage>();
        Configure<SettingsViewModel, SettingsPage>();
    }

    public Type GetPageType(string key)
    {
        Type? pageType;
        lock (_pages)
        {
            if (!_pages.TryGetValue(key, out pageType))
            {
                throw new ArgumentException($"Page not found: {key}. Did you forget to call PageService.Configure?");
            }
        }

        return pageType;
    }

    private void Configure<VM, V>()
        where VM : ObservableObject
        where V : Page
    {
        lock (_pages)
        {
            var key = typeof(VM).FullName!;
            if (_pages.ContainsKey(key))
            {
                throw new ArgumentException($"The key {key} is already configured in PageService");
            }

            var type = typeof(V);
            if (_pages.ContainsValue(type))
            {
                throw new ArgumentException($"This type is already configured with key {_pages.First(p => p.Value == type).Key}");
            }

            _pages.Add(key, type);

            if (type.GetField("IsPageHidden", BindingFlags.Public | BindingFlags.Static)?.GetValue(null) is bool isPageHidden && isPageHidden)
            {
                _hiddenPages.Add(key, type);
            }

            var _page_name = type.GetField("PageName", BindingFlags.Public | BindingFlags.Static)?.GetValue(null) as string;
            _pageNames.Add(key, (_page_name ?? type.Name).Replace("Page", string.Empty));
        }
    }

    public Dictionary<string, string> GetPageKeys()
    {
        lock (_pages)
        {
            var result = new Dictionary<string, string>();
            foreach (var page in _pageNames)
            {
                if (!_hiddenPages.ContainsKey(page.Key))
                {
                    result.Add(page.Key, page.Value);
                }
            }
            return result;
        }
    }
}
