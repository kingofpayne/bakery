/**
 * Copyright (C) 2010, 2011, 2012, 2013
 * Olivier Hériveaux.
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Lesser General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 * @author Olivier Hériveaux
 */


#ifndef _BAKERY_RECIPE_OR_DATA_HPP_
#define _BAKERY_RECIPE_OR_DATA_HPP_


#include "rec/recipe.hpp"
#include "dat/data.hpp"


namespace bakery {


/**
 * Class which holds either a recipe or a data, or nothing.
 */
class recipe_or_data_t
{
    public:
        recipe_or_data_t();

        void set_recipe(const rec::recipe &);
        void set_data(const dat::data &);
        bool is_recipe() const;
        bool is_data() const;
        rec::recipe & get_recipe();
        dat::data & get_data();

    private:
        boost::variant<
            int, /* int to hold nothing. */
            rec::recipe,
            dat::data
        > value;
};


} /* namespace bakery */


#endif

