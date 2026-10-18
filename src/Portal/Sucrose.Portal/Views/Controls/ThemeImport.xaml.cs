using System.Windows;
using System.Windows.Controls;
using System.Windows.Media;
using Wpf.Ui.Controls;
using SPMI = Sucrose.Portal.Manage.Internal;
using SRER = Sucrose.Resources.Extension.Resources;
using SSDECT = Sucrose.Shared.Dependency.Enum.CompatibilityType;

namespace Sucrose.Portal.Views.Controls
{
    /// <summary>
    /// ThemeImport.xaml etkileşim mantığı
    /// </summary>
    public partial class ThemeImport : ContentDialog, IDisposable
    {
        internal List<SSDECT> Types = [];
        internal List<string> Messages = [];

        public ThemeImport() : base(SPMI.ContentDialogService.GetDialogHostEx())
        {
            InitializeComponent();
        }

        private async void ContentDialog_Loaded(object sender, RoutedEventArgs e)
        {
            if (Messages.Any())
            {
                MessageCard.Visibility = Visibility.Visible;

                Messages.ForEach(Message =>
                {
                    bool Critical = Types.Any(Type => Message.Contains($"'{Type}'.") || Message.Contains($"'{Type}'।") || Message.Contains($"'{Type}'。"));

                    Brush Status = SRER.GetResource<Brush>(Critical ? "SystemFillColorCriticalBrush" : "SystemFillColorSuccessBrush");

                    Grid Row = new();

                    if (ThemeImportMessage.Children.Count > 0)
                    {
                        Row.Margin = new Thickness(0, 6, 0, 0);
                    }

                    Row.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
                    Row.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });

                    SymbolIcon Icon = new(Critical ? SymbolRegular.ErrorCircle24 : SymbolRegular.CheckmarkCircle24)
                    {
                        FontSize = 16,
                        Foreground = Status,
                        VerticalAlignment = VerticalAlignment.Top
                    };

                    Grid.SetColumn(Icon, 0);

                    Row.Children.Add(Icon);

                    Wpf.Ui.Controls.TextBlock Text = new()
                    {
                        Text = Message,
                        TextAlignment = TextAlignment.Left,
                        TextWrapping = TextWrapping.WrapWithOverflow,
                        VerticalAlignment = VerticalAlignment.Center,
                        HorizontalAlignment = HorizontalAlignment.Left,
                        Foreground = Status,
                        Margin = new Thickness(8, 0, 0, 0)
                    };

                    Grid.SetColumn(Text, 1);

                    Row.Children.Add(Text);

                    ThemeImportMessage.Children.Add(Row);
                });
            }

            if (Types.Any())
            {
                TypeCard.Visibility = Visibility.Visible;

                Types.ForEach(Type =>
                {
                    Brush Status = SRER.GetResource<Brush>("SystemFillColorCriticalBrush");

                    Grid Row = new();

                    if (ThemeImportType.Children.Count > 0)
                    {
                        Row.Margin = new Thickness(0, 6, 0, 0);
                    }

                    Row.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
                    Row.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });

                    SymbolIcon Icon = new(SymbolRegular.ErrorCircle24)
                    {
                        FontSize = 16,
                        Foreground = Status,
                        VerticalAlignment = VerticalAlignment.Top
                    };

                    Grid.SetColumn(Icon, 0);

                    Row.Children.Add(Icon);

                    Wpf.Ui.Controls.TextBlock Text = new()
                    {
                        TextAlignment = TextAlignment.Left,
                        TextWrapping = TextWrapping.WrapWithOverflow,
                        VerticalAlignment = VerticalAlignment.Center,
                        HorizontalAlignment = HorizontalAlignment.Left,
                        Foreground = Status,
                        Text = $"{Type}: {SRER.GetValue("Portal", "Enum", "CompatibilityType", $"{Type}")}",
                        Margin = new Thickness(8, 0, 0, 0)
                    };

                    Grid.SetColumn(Text, 1);

                    Row.Children.Add(Text);

                    ThemeImportType.Children.Add(Row);
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