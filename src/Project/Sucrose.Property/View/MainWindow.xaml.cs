using HandyControl.Controls;
using HandyControl.Themes;
using HandyControl.Tools;
using System.Reflection;
using System.Resources;
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
            // Pencerenin maksimum yüksekliğini ekran yüksekliği ile sınırla
            double screenHeight = SystemParameters.PrimaryScreenHeight;

            // Pencerenin sağ alt köşede açılması için konumunu ayarla
            double screenWidth = SystemParameters.PrimaryScreenWidth;

            AnchorStyles Anchor = SWHWTR.GetAnchorStyle();
            Rectangle TaskbarPosition = SWHWTR.GetPosition();
            Rectangle TaskbarCoordinates = SWHWTR.GetCoordonates();

            switch (Anchor)
            {
                case AnchorStyles.Top:
                    MaxHeight = screenHeight - TaskbarCoordinates.Height - 20;

                    Left = screenWidth - Width - 10;
                    Top = screenHeight - Height - 10;
                    break;
                case AnchorStyles.Bottom:
                    MaxHeight = screenHeight - TaskbarCoordinates.Height - 20;

                    Left = screenWidth - Width - 10;
                    Top = TaskbarPosition.Top - Height - 10;
                    break;
                case AnchorStyles.Left:
                    MaxHeight = screenHeight - 20;

                    Left = screenWidth - Width - 10;
                    Top = screenHeight - Height - 10;
                    break;
                case AnchorStyles.Right:
                    MaxHeight = screenHeight - 20;

                    Left = TaskbarPosition.Left - Width - 10;
                    Top = screenHeight - Height - 10;
                    break;
                default:
                    MaxHeight = screenHeight - 20;

                    Left = screenWidth - Width - 10;
                    Top = screenHeight - Height - 10;
                    break;
            }
        }

        private void DropDownButton_Click(object sender, RoutedEventArgs e)
        {
            Popup.IsOpen = true;
        }

        private void ColorSliders_ColorChanged(object sender, RoutedEventArgs e)
        {
            PCP.SelectedColor = CS.SelectedColor;
        }

        private void OpenButton_Click(object sender, RoutedEventArgs e)
        {
            //Popup2.IsOpen = true;

            HandyControl.Controls.ColorPicker picker = SingleOpenHelper.CreateControl<HandyControl.Controls.ColorPicker>();

            PopupWindow window = new()
            {
                PopupElement = picker,
                WindowStartupLocation = WindowStartupLocation.Manual,
                AllowsTransparency = true,
                WindowStyle = WindowStyle.None,
                MinWidth = 0,
                MinHeight = 0,
                Title = "Sucrose Property Color Picker"
            };

            picker.Confirmed += (s, ee) => { PCP.SelectedColor = ee.Info; window.Close(); };
            picker.SelectedColorChanged += (s, ee) => { PCP.SelectedColor = ee.Info; };
            picker.Canceled += delegate { window.Close(); };

            window.Show(OpenButton, false);
        }

        private void ColorPicker_SelectedColorChanged(object sender, HandyControl.Data.FunctionEventArgs<System.Windows.Media.Color> e)
        {
            PCP.SelectedColor = e.Info;
        }
    }
}