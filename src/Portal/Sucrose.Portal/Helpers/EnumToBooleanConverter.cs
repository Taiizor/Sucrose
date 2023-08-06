using System.Globalization;
using System.Windows.Data;

namespace Sucrose.Portal.Helpers
{
    internal class EnumToBooleanConverter : IValueConverter
    {
        public EnumToBooleanConverter()
        {
        }

        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            if (parameter is not string enumString)
            {
                throw new ArgumentException("ExceptionEnumToBooleanConverterParameterMustBeAnEnumName");
            }

            if (!Enum.IsDefined(typeof(Wpf.Ui.Appearance.ApplicationTheme), value))
            {
                throw new ArgumentException("ExceptionEnumToBooleanConverterValueMustBeAnEnum");
            }

            object enumValue = Enum.Parse(typeof(Wpf.Ui.Appearance.ApplicationTheme), enumString);

            return enumValue.Equals(value);
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            if (parameter is not string enumString)
            {
                throw new ArgumentException("ExceptionEnumToBooleanConverterParameterMustBeAnEnumName");
            }

            return Enum.Parse(typeof(Wpf.Ui.Appearance.ApplicationTheme), enumString);
        }
    }
}