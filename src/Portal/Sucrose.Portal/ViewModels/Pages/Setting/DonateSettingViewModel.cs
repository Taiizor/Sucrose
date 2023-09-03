using CommunityToolkit.Mvvm.ComponentModel;
using System.Windows;
using Wpf.Ui.Controls;

namespace Sucrose.Portal.ViewModels.Pages
{
    public partial class DonateSettingViewModel : ObservableObject, INavigationAware, IDisposable
    {
        [ObservableProperty]
        private List<UIElement> _Contents = new();

        private bool _isInitialized;

        public DonateSettingViewModel()
        {
            if (!_isInitialized)
            {
                InitializeViewModel();
            }
        }

        private void InitializeViewModel()
        {
            //

            _isInitialized = true;
        }

        public void OnNavigatedTo()
        {
            //
        }

        public void OnNavigatedFrom()
        {
            Dispose();
        }

        public void Dispose()
        {
            Contents.Clear();

            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}