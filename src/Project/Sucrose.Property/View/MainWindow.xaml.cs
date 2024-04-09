using HandyControl.Controls;
using HandyControl.Themes;
using HandyControl.Tools;
using System.Windows;
using System.Windows.Input;
using Wpf.Ui.Appearance;
using Wpf.Ui.Controls;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SPMM = Sucrose.Property.Manage.Manager;
using SWHWTR = Skylark.Wing.Helper.WindowsTaskbar;

namespace Sucrose.Property.View
{
    /// <summary>
    /// Interaction logic for MainWindow.xaml
    /// </summary>
    public partial class MainWindow : FluentWindow
    {
        public MainWindow()
        {
            InitializeComponent();

            if (SPMM.ThemeType == SEWTT.Dark)
            {
                ApplicationThemeManager.Apply(Wpf.Ui.Appearance.ApplicationTheme.Dark);
                ThemeManager.Current.ApplicationTheme = HandyControl.Themes.ApplicationTheme.Dark;
            }
            else
            {
                ApplicationThemeManager.Apply(Wpf.Ui.Appearance.ApplicationTheme.Light);
                ThemeManager.Current.ApplicationTheme = HandyControl.Themes.ApplicationTheme.Light;
            }

            Loaded += MainWindow_Loaded;
        }

        private void TitleBar_MouseDown(object sender, MouseButtonEventArgs e)
        {
            if (e.ChangedButton == MouseButton.Left)
            {
                DragMove();
            }
        }

        private void MainWindow_Loaded(object sender, RoutedEventArgs e)
        {
            double ScreenWidth = SystemParameters.PrimaryScreenWidth;
            double ScreenHeight = SystemParameters.PrimaryScreenHeight;

            AnchorStyles Anchor = SWHWTR.GetAnchorStyle();
            Rectangle TaskbarPosition = SWHWTR.GetPosition();
            Rectangle TaskbarCoordinates = SWHWTR.GetCoordonates();

            switch (Anchor)
            {
                case AnchorStyles.Top:
                    MaxHeight = ScreenHeight - TaskbarCoordinates.Height - 20;

                    Left = ScreenWidth - Width - 10;
                    Top = ScreenHeight - Height - 10;
                    break;
                case AnchorStyles.Bottom:
                    MaxHeight = ScreenHeight - TaskbarCoordinates.Height - 20;

                    Left = ScreenWidth - Width - 10;
                    Top = TaskbarPosition.Top - Height - 10;
                    break;
                case AnchorStyles.Left:
                    MaxHeight = ScreenHeight - 20;

                    Left = ScreenWidth - Width - 10;
                    Top = ScreenHeight - Height - 10;
                    break;
                case AnchorStyles.Right:
                    MaxHeight = ScreenHeight - 20;

                    Left = TaskbarPosition.Left - Width - 10;
                    Top = ScreenHeight - Height - 10;
                    break;
                default:
                    MaxHeight = ScreenHeight - 20;

                    Left = ScreenWidth - Width - 10;
                    Top = ScreenHeight - Height - 10;
                    break;
            }
        }

        private void DropDownButton_Click(object sender, RoutedEventArgs e)
        {
            ColorPicker picker = SingleOpenHelper.CreateControl<ColorPicker>();

            System.Windows.Media.Color Temp = SCB.Color;

            picker.SelectedBrush = new(Temp);
            picker.UseLayoutRounding = true;

            PopupWindow window = new()
            {
                PopupElement = picker,
                WindowStartupLocation = WindowStartupLocation.Manual,
                AllowsTransparency = true,
                WindowStyle = WindowStyle.None,
                ResizeMode = ResizeMode.NoResize,
                UseLayoutRounding = true,
                ShowBorder = true,
                Topmost = true,
                Title = "Sucrose Property Color Picker"
            };

            picker.Confirmed += (s, ee) => { SCB.Color = ee.Info; window.Close(); };
            picker.SelectedColorChanged += (s, ee) => { SCB.Color = ee.Info; };
            picker.Canceled += delegate { SCB.Color = Temp; window.Close(); };

            window.Show(Refresh, false);
        }
    }
}