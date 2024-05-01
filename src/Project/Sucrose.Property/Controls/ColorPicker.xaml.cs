using HandyControl.Controls;
using HandyControl.Themes;
using HandyControl.Tools;
using System.Windows;
using Wpf.Ui.Controls;
using Control = System.Windows.Controls.Control;
using CPicker = HandyControl.Controls.ColorPicker;
using DrawingColor = System.Drawing.Color;
using MediaColor = System.Windows.Media.Color;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SPHP = Sucrose.Property.Helper.Properties;
using SPMM = Sucrose.Property.Manage.Manager;
using SRER = Sucrose.Resources.Extension.Resources;
using SSDMM = Sucrose.Shared.Dependency.Manage.Manager;
using SSECCE = Skylark.Standard.Extension.Color.ColorExtension;
using SSTMCPM = Sucrose.Shared.Theme.Model.ColorPickerModel;
using SWHWT = Skylark.Wing.Helper.WindowsTheme;
using ToolTip = System.Windows.Controls.ToolTip;
using UserControl = System.Windows.Controls.UserControl;

namespace Sucrose.Property.Controls
{
    /// <summary>
    /// ColorPicker.xaml etkileşim mantığı
    /// </summary>
    public partial class ColorPicker : UserControl
    {
        private Control Control { get; set; }

        public ColorPicker(string Key, SSTMCPM Data, Control Control)
        {
            InitializeComponent();

            this.Control = Control;

            InitializeData(Key, Data);

            if (SPMM.BackdropType == WindowBackdropType.Auto)
            {
                if (SWHWT.GetTheme() == SEWTT.Dark)
                {
                    ThemeManager.Current.ApplicationTheme = ApplicationTheme.Dark;
                }
                else
                {
                    ThemeManager.Current.ApplicationTheme = ApplicationTheme.Light;
                }
            }
            else
            {
                if (SSDMM.ThemeType == SEWTT.Dark)
                {
                    ThemeManager.Current.ApplicationTheme = ApplicationTheme.Dark;
                }
                else
                {
                    ThemeManager.Current.ApplicationTheme = ApplicationTheme.Light;
                }
            }
        }

        private void InitializeData(string Key, SSTMCPM Data)
        {
            Label.Text = Data.Text;
            DrawingColor Color = SSECCE.HexToColor(Data.Value);
            Component.Color = MediaColor.FromArgb(Color.A, Color.R, Color.G, Color.B);

            DropDownButton.Click += (s, e) => DropDownButton_Click(Key, Data);

            if (!string.IsNullOrEmpty(Data.Help))
            {
                ToolTip HelpTip = new()
                {
                    Content = Data.Help
                };

                Container.ToolTip = HelpTip;
            }
        }

        private void DropDownButton_Click(string Key, SSTMCPM Data)
        {
            CPicker Picker = SingleOpenHelper.CreateControl<CPicker>();

            MediaColor UndoColor = Component.Color;

            Picker.SelectedBrush = new(UndoColor);
            Picker.UseLayoutRounding = false;

            PopupWindow PopupWindow = new()
            {
                Title = SRER.GetValue("Property", "ColorPicker", "Popup", "Title"),
                WindowStartupLocation = WindowStartupLocation.Manual,
                ResizeMode = ResizeMode.NoResize,
                WindowStyle = WindowStyle.None,
                AllowsTransparency = true,
                UseLayoutRounding = true,
                PopupElement = Picker,
                ShowBorder = true,
                Topmost = true
            };

            Picker.Confirmed += (s, e) =>
            {
                Data.Value = e.Info.ToString();

                Component.Color = e.Info;

                SPHP.Change(Key, Data);

                PopupWindow.Close();
            };

            Picker.SelectedColorChanged += (s, e) =>
            {
                Component.Color = e.Info;
            };

            Picker.Canceled += delegate
            {
                Component.Color = UndoColor;

                PopupWindow.Close();
            };

            PopupWindow.Show(Control, false);
        }
    }
}