using System.Windows;
using System.Windows.Media;
using Wpf.Ui.Controls;
using SPMI = Sucrose.Portal.Manage.Internal;
using SSRER = Sucrose.Shared.Resources.Extension.Resources;
using SSDECT = Sucrose.Shared.Dependency.Enum.CompatibilityType;

namespace Sucrose.Portal.Views.Controls
{
    /// <summary>
    /// ThemeImport.xaml etkileşim mantığı
    /// </summary>
    public partial class ThemeImport : ContentDialog, IDisposable
    {
        internal List<SSDECT> Types = new();
        internal string Info = string.Empty;

        public ThemeImport() : base(SPMI.ContentDialogService.GetContentPresenter())
        {
            InitializeComponent();
        }

        private async void ContentDialog_Loaded(object sender, RoutedEventArgs e)
        {
            ThemeImportInfo.Text = Info;

            if (Types.Any())
            {
                TypeCard.Visibility = Visibility.Visible;

                Types.ForEach(Type =>
                {
                    ThemeImportType.Children.Add(new TextBlock
                    {
                        Text = $"{Type}: Bu şu anlama geliyor",
                        TextAlignment = TextAlignment.Left,
                        TextWrapping = TextWrapping.WrapWithOverflow,
                        VerticalAlignment = VerticalAlignment.Center,
                        HorizontalAlignment = HorizontalAlignment.Center,
                        Foreground = SSRER.GetResource<Brush>("SystemFillColorCriticalBrush")
                    });
                });
            }

            await Task.Delay(10);

            Panel.MinHeight = 0;
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}