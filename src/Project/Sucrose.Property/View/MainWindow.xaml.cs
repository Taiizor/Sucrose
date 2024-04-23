using System.Windows;
using System.Windows.Input;
using Wpf.Ui.Appearance;
using Wpf.Ui.Controls;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SPMI = Sucrose.Property.Manage.Internal;
using SPMM = Sucrose.Property.Manage.Manager;
using SSTMCM = Sucrose.Shared.Theme.Model.ControlModel;
using SWHWTR = Skylark.Wing.Helper.WindowsTaskbar;
using SPCB = Sucrose.Property.Controls.Button;
using SPCCB = Sucrose.Property.Controls.CheckBox;
using SPCCP = Sucrose.Property.Controls.ColorPicker;
using SPCDD = Sucrose.Property.Controls.DropDown;
using SPCFDD = Sucrose.Property.Controls.FileDropDown;
using SPCL = Sucrose.Property.Controls.Label;
using SPCNB = Sucrose.Property.Controls.NumberBox;
using SPCPB = Sucrose.Property.Controls.PasswordBox;
using SPCS = Sucrose.Property.Controls.Slider;
using SPCTB = Sucrose.Property.Controls.TextBox;
using SSTMSM = Sucrose.Shared.Theme.Model.SliderModel;
using SSTMLM = Sucrose.Shared.Theme.Model.LabelModel;
using SSTMBM = Sucrose.Shared.Theme.Model.ButtonModel;
using SSTMCBM = Sucrose.Shared.Theme.Model.CheckBoxModel;
using SSTMCPM = Sucrose.Shared.Theme.Model.ColorPickerModel;
using SSTMDDM = Sucrose.Shared.Theme.Model.DropDownModel;
using SSTMFDDM = Sucrose.Shared.Theme.Model.FileDropDownModel;
using SSTMFTBM = Sucrose.Shared.Theme.Model.TextBoxModel;
using SSTMFNBM = Sucrose.Shared.Theme.Model.NumberBoxModel;
using SSTMFPBM = Sucrose.Shared.Theme.Model.PasswordBoxModel;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;
using SWHWT = Skylark.Wing.Helper.WindowsTheme;

namespace Sucrose.Property.View
{
    /// <summary>
    /// Interaction logic for MainWindow.xaml
    /// </summary>
    public partial class MainWindow : FluentWindow
    {
        public MainWindow()
        {
            DataContext = this;

            Visibility = Visibility.Collapsed;

            InitializeComponent();

            if (SPMM.BackdropType == WindowBackdropType.Auto)
            {
                if (SWHWT.GetTheme() == SEWTT.Dark)
                {
                    ApplicationThemeManager.Apply(ApplicationTheme.Dark);
                }
                else
                {
                    ApplicationThemeManager.Apply(ApplicationTheme.Light);
                }
            }
            else
            {
                if (SPMM.ThemeType == SEWTT.Dark)
                {
                    ApplicationThemeManager.Apply(ApplicationTheme.Dark);
                }
                else
                {
                    ApplicationThemeManager.Apply(ApplicationTheme.Light);
                }
            }

            Container_Controls();

            Loaded += MainWindow_Loaded;

            WindowBackdropType = GetWindowBackdropType();
        }

        private void Container_Controls()
        {
            Container.Children.Clear();

            foreach (KeyValuePair<string, SSTMCM> Pair in SPMI.Properties.PropertyList)
            {
                Container.Children.Add(Pair.Value.Type.ToLower() switch
                {
                    "label" => new SPCL(Pair.Value as SSTMLM),
                    //"button" => new SPCB(Pair.Value),
                    //"slider" => new SPCS(Pair.Value),
                    //"textbox" => new SPCTB(Pair.Value),
                    //"checkbox" => new SPCCB(Pair.Value),
                    //"dropdown" => new SPCDD(Pair.Value),
                    //"numberbox" => new SPCNB(Pair.Value),
                    //"colorpicker" => new SPCCP(Pair.Value),
                    //"passwordbox" => new SPCPB(Pair.Value),
                    //"filedropdown" => new SPCFDD(Pair.Value),
                    _ => throw new NotSupportedException($"Control type '{Pair.Value.Type}' is not supported."),
                });
            }
        }

        private static WindowBackdropType GetWindowBackdropType()
        {
            if (WindowBackdrop.IsSupported(SPMM.BackdropType))
            {
                return SPMM.BackdropType;
            }
            else
            {
                return SPMM.DefaultBackdropType;
            }
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

            Visibility = Visibility.Visible;
        }
    }
}