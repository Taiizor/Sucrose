using HandyControl.Controls;
using HandyControl.Themes;
using HandyControl.Tools;
using System.Windows;
using Wpf.Ui.Controls;
using Control = System.Windows.Controls.Control;
using CPicker = HandyControl.Controls.ColorPicker;
using DColor = System.Drawing.Color;
using MColor = System.Windows.Media.Color;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SPMM = Sucrose.Property.Manage.Manager;
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
        public Control Control { get; set; }

        public ColorPicker(SSTMCPM Data, Control Control)
        {
            InitializeComponent();

            InitializeData(Data);

            this.Control = Control;

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
                if (SPMM.ThemeType == SEWTT.Dark)
                {
                    ThemeManager.Current.ApplicationTheme = ApplicationTheme.Dark;
                }
                else
                {
                    ThemeManager.Current.ApplicationTheme = ApplicationTheme.Light;
                }
            }
        }

        private void InitializeData(SSTMCPM Data)
        {
            DColor Color = SSECCE.HexToColor(Data.Value);
            Component.Color = MColor.FromArgb(Color.A, Color.R, Color.G, Color.B);

            if (!string.IsNullOrEmpty(Data.Help))
            {
                ToolTip HelpTip = new()
                {
                    Content = Data.Help
                };

                Container.ToolTip = HelpTip;
            }
        }

        private void DropDownButton_Click(object sender, RoutedEventArgs e)
        {
            CPicker Picker = SingleOpenHelper.CreateControl<CPicker>();

            MColor Temp = Component.Color;

            Picker.SelectedBrush = new(Temp);
            Picker.UseLayoutRounding = true;

            PopupWindow PWindow = new()
            {
                PopupElement = Picker,
                WindowStartupLocation = WindowStartupLocation.Manual,
                AllowsTransparency = true,
                WindowStyle = WindowStyle.None,
                ResizeMode = ResizeMode.NoResize,
                UseLayoutRounding = true,
                ShowBorder = true,
                Topmost = true,
                Title = "Sucrose Property Color Picker"
            };

            Picker.Confirmed += (s, ee) => { Component.Color = ee.Info; PWindow.Close(); };
            Picker.SelectedColorChanged += (s, ee) => { Component.Color = ee.Info; };
            Picker.Canceled += delegate { Component.Color = Temp; PWindow.Close(); };

            PWindow.Show(Control, false);
        }
    }
}